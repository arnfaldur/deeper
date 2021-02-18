use cgmath::{prelude::*, Vector2};
use crossbeam_channel::Receiver;
use legion::systems::{Builder, CommandBuffer};
use legion::world::{Event, SubWorld};
use legion::*;
use nalgebra::Isometry2;
use ncollide2d::shape::ShapeHandle;
use nphysics2d::force_generator::DefaultForceGeneratorSet;
use nphysics2d::joint::DefaultJointConstraintSet;
use nphysics2d::ncollide2d::shape::{Ball, Cuboid};
use nphysics2d::object::{
    BodyPartHandle, BodyStatus, ColliderDesc, DefaultBodySet, DefaultColliderSet, RigidBodyDesc,
};
use nphysics2d::world::{DefaultGeometricalWorld, DefaultMechanicalWorld};

use crate::components::*;

pub(crate) trait PhysicsBuilderExtender {
    fn add_physics_systems(&mut self, world: &mut World, resources: &mut Resources) -> &mut Self;
}
impl PhysicsBuilderExtender for Builder {
    fn add_physics_systems(&mut self, world: &mut World, resources: &mut Resources) -> &mut Self {
        resources.insert(PhysicsResource::default());
        let (sender, _receiver) = crossbeam_channel::unbounded::<Event>();
        world.subscribe(
            sender,
            component::<BodyHandle>() | component::<ColliderHandle>(),
        );
        return self
            .add_system(validate_physics_entities_system())
            .add_system(make_body_handles_system())
            .add_system(remove_body_handles_system())
            // .add_system(flush_command_buffer_system())
            .add_system(make_collider_handles_system())
            .add_system(remove_collider_handles_system())
            //.add_system(handle_entity_removal_system(receiver))
            // .add_system(flush_command_buffer_system())
            .add_system(entity_world_to_physics_world_system())
            .add_system(step_physics_world_system())
            .add_system(physics_world_to_entity_world_system());
        //      .add_system(movement_system());
    }
}
struct PhysicsResource {
    mechanical_world: DefaultMechanicalWorld<f32>,
    geometrical_world: DefaultGeometricalWorld<f32>,
    bodies: DefaultBodySet<f32>,
    colliders: DefaultColliderSet<f32>,
    joint_constraints: DefaultJointConstraintSet<f32>,
    force_generators: DefaultForceGeneratorSet<f32>,
}
impl PhysicsResource {
    fn step(&mut self) {
        self.mechanical_world.step(
            &mut self.geometrical_world,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.joint_constraints,
            &mut self.force_generators,
        )
    }
}
impl Default for PhysicsResource {
    fn default() -> Self {
        PhysicsResource {
            mechanical_world: DefaultMechanicalWorld::new(
                nalgebra::zero::<nalgebra::Vector2<f32>>(),
            ),
            geometrical_world: DefaultGeometricalWorld::new(),
            bodies: DefaultBodySet::new(),
            colliders: DefaultColliderSet::new(),
            joint_constraints: DefaultJointConstraintSet::new(),
            force_generators: DefaultForceGeneratorSet::new(),
        }
    }
}

fn validate_physics_entities_system() -> impl ::legion::systems::Runnable {
    SystemBuilder::new("validate_physics_entities")
        .read_component::<WorldPosition>()
        .read_component::<Velocity>()
        .read_component::<Speed>()
        .read_component::<Acceleration>()
        .read_component::<Orientation>()
        .read_component::<DynamicBody>()
        .read_component::<StaticBody>()
        .write_component::<Force>()
        .build(move |cmd, world, resources, query| {
            validate_physics_entities(world, cmd);
        })
}

#[allow(dead_code)]
fn validate_physics_entities(world: &mut SubWorld, _commands: &mut CommandBuffer) {
    let mut query = <(
        Entity,
        TryRead<WorldPosition>,
        TryRead<Velocity>,
        TryRead<Force>,
        TryRead<Orientation>,
        TryRead<StaticBody>,
    )>::query()
    .filter(component::<DynamicBody>());
    for (_ent, pos, vel, _frc, ori, sta) in query.iter(world) {
        if pos.is_none() {
            panic!("missing Position in DynamicBody");
        } else if vel.is_none() {
            panic!("missing Velocity in DynamicBody");
            // TODO: decide if Force should be in our repertoire
            // } else if frc.is_none() {
            //     commands.add_component(*ent, Force::default());
        } else if ori.is_none() {
            panic!("missing Orientation in DynamicBody");
        } else if sta.is_some() {
            // this is an awfully DynamicBody normative perspective
            panic!("There's a naughty StaticBody that really feels DynamicBody inside.");
        }
    }
    let mut query = <(
        Entity,
        TryRead<WorldPosition>,
        TryRead<Velocity>,
        TryRead<Speed>,
        TryRead<Acceleration>,
        TryRead<Force>,
        TryRead<Orientation>,
    )>::query()
    .filter(component::<StaticBody>());
    for (_ent, pos, vel, spd, acc, frc, _ori) in query.iter(world) {
        if pos.is_none() {
            panic!("missing Position in StaticBody");
        } else if vel.is_some() {
            panic!("StaticBody can't have a Velocity component");
        } else if spd.is_some() {
            panic!("StaticBody can't have a Speed component");
        } else if acc.is_some() {
            panic!("StaticBody can't have a Acceleration component");
        } else if frc.is_some() {
            panic!("StaticBody can't have a Force component");
            // } else if ori.is_none() {
            //     panic!("missing Orientation in StaticBody");
        }
    }
}

fn make_body_handles_system() -> impl ::legion::systems::Runnable {
    SystemBuilder::new("make_body_handles")
        .write_resource::<PhysicsResource>()
        .with_query(
            <(
                Entity,
                TryRead<DynamicBody>,
                TryRead<StaticBody>,
                TryRead<DisabledBody>,
                TryRead<WorldPosition>,
            )>::query()
            .filter(
                (component::<DynamicBody>()
                    | component::<StaticBody>()
                    | component::<DisabledBody>())
                    & !component::<BodyHandle>(),
            ),
        )
        .build(move |cmd, world, resources, query| {
            let (mut for_query, mut world) = world.split_for_query(query);
            let for_query = &mut for_query;
            query.for_each_mut(for_query, |components| {
                make_body_handles(
                    &mut world,
                    cmd,
                    &mut *resources,
                    components.0,
                    components.1,
                    components.2,
                    components.3,
                    components.4,
                );
            });
        })
}

#[allow(dead_code)]
fn make_body_handles(
    _world: &mut SubWorld,
    commands: &mut legion::systems::CommandBuffer,
    physics: &mut PhysicsResource,
    entity: &Entity,
    dynamic: Option<&DynamicBody>,
    stat: Option<&StaticBody>,
    disabled: Option<&DisabledBody>,
    position: Option<&WorldPosition>,
) {
    let body = if let Some(dyna) = dynamic {
        RigidBodyDesc::<f32>::new()
            .status(BodyStatus::Dynamic)
            .gravity_enabled(false)
            .mass(dyna.mass)
    } else if stat.is_some() {
        RigidBodyDesc::<f32>::new()
            .status(BodyStatus::Static)
            .position(Isometry2::new(
                c2n(position.unwrap_or(&WorldPosition(cgmath::vec2(0., 0.))).0),
                0.,
            ))
    } else if disabled.is_some() {
        RigidBodyDesc::<f32>::new().status(BodyStatus::Disabled)
    } else {
        unreachable!() // the filter should take care of this
    };
    let handle = BodyHandle(physics.bodies.insert(body.build()));
    commands.add_component(*entity, handle);
}

fn remove_body_handles_system() -> impl ::legion::systems::Runnable {
    SystemBuilder::new("remove_body_handles")
        .write_resource::<PhysicsResource>()
        .with_query(
            <(::legion::Entity, ::legion::Read<BodyHandle>)>::query().filter(
                !component::<DynamicBody>()
                    & !component::<StaticBody>()
                    & !component::<DisabledBody>(),
            ),
        )
        .build(move |cmd, world, resources, query| {
            let for_query = world;
            query.for_each_mut(for_query, |components| {
                remove_body_handles(cmd, &mut *resources, components.0, components.1);
            });
        })
}

#[allow(dead_code)]
fn remove_body_handles(
    commands: &mut legion::systems::CommandBuffer,
    physics: &mut PhysicsResource,
    entity: &Entity,
    handle: &BodyHandle,
) {
    physics.bodies.remove(handle.0);
    commands.remove_component::<BodyHandle>(*entity);
}

fn flush_command_buffer_system() -> impl ::legion::systems::Runnable {
    SystemBuilder::new("flush_command_buffer")
        .with_query(<(::legion::Write<World>)>::query())
        .build(move |cmd, world, resources, query| {
            let for_query = world;
            query.for_each_mut(for_query, |components| {
                flush_command_buffer(components, cmd);
            });
        })
}

#[allow(dead_code)]
fn flush_command_buffer(world: &mut World, commands: &mut legion::systems::CommandBuffer) {
    commands.flush(world);
}

fn make_collider_handles_system() -> impl ::legion::systems::Runnable {
    SystemBuilder::new("make_collider_handles")
        .write_resource::<PhysicsResource>()
        .with_query(
            <(
                Entity,
                Read<BodyHandle>,
                TryRead<CircleCollider>,
                TryRead<SquareCollider>,
            )>::query()
            .filter(
                (component::<CircleCollider>() | component::<SquareCollider>())
                    & !component::<ColliderHandle>(),
            ),
        )
        .build(move |cmd, world, resources, query| {
            let (mut for_query, world) = world.split_for_query(query);
            let for_query = &mut for_query;
            query.for_each_mut(for_query, |components| {
                make_collider_handles(
                    &world,
                    cmd,
                    &mut *resources,
                    components.0,
                    components.1,
                    components.2,
                    components.3,
                );
            });
        })
}

#[allow(dead_code)]
fn make_collider_handles(
    _world: &SubWorld,
    commands: &mut legion::systems::CommandBuffer,
    physics: &mut PhysicsResource,
    entity: &Entity,
    body_handle: &BodyHandle,
    circle: Option<&CircleCollider>,
    square: Option<&SquareCollider>,
) {
    let shape_handle = if let Some(c) = circle {
        ShapeHandle::new(Ball::new(c.radius))
    } else if let Some(s) = square {
        let side_length = s.side_length / 2.0;
        let sides_vec = nalgebra::Vector2::new(side_length, side_length);
        ShapeHandle::new(Cuboid::new(sides_vec))
    } else {
        unreachable!() // the filter should prevent this
    };
    let collider = ColliderDesc::<f32>::new(shape_handle);
    let handle = ColliderHandle(
        physics
            .colliders
            .insert(collider.build(BodyPartHandle(body_handle.0, 0))),
    );
    commands.add_component(*entity, handle);
}

fn remove_collider_handles_system() -> impl ::legion::systems::Runnable {
    SystemBuilder::new("remove_collider_handles")
        .write_resource::<PhysicsResource>()
        .with_query(
            <(Entity, Read<ColliderHandle>)>::query()
                .filter(!component::<CircleCollider>() & !component::<SquareCollider>()),
        )
        .build(move |cmd, world, resources, query| {
            let for_query = world;
            query.for_each_mut(for_query, |components| {
                remove_collider_handles(cmd, &mut *resources, components.0, components.1);
            });
        })
}

#[allow(dead_code)]
fn remove_collider_handles(
    commands: &mut legion::systems::CommandBuffer,
    physics: &mut PhysicsResource,
    entity: &Entity,
    collider_handle: &ColliderHandle,
) {
    physics.colliders.remove(collider_handle.0);
    commands.remove_component::<ColliderHandle>(*entity);
}

fn handle_entity_removal_system(state_0: Receiver<Event>) -> impl ::legion::systems::Runnable {
    SystemBuilder::new("handle_entity_removal")
        .read_component::<BodyHandle>()
        .read_component::<ColliderHandle>()
        .write_resource::<PhysicsResource>()
        .build(move |cmd, world, resources, query| {
            handle_entity_removal(world, cmd, &mut *resources, &state_0);
        })
}

#[allow(dead_code)]
fn handle_entity_removal(
    world: &SubWorld,
    commands: &mut legion::systems::CommandBuffer,
    physics: &mut PhysicsResource,
    receiver: &Receiver<Event>,
) {
    for event in receiver.try_iter() {
        match event {
            Event::EntityRemoved(entity, _arch) => {
                // FIXME: find a better solution or figure out how to know when an entity is being completely removed
                if let Some(body_handle) = world
                    .entry_ref(entity)
                    .ok()
                    .map(|e| e.into_component::<BodyHandle>().ok())
                    .flatten()
                {
                    physics.bodies.remove(body_handle.0);
                    commands.remove_component::<BodyHandle>(entity);
                }
                if let Some(collider_handle) = world
                    .entry_ref(entity)
                    .ok()
                    .map(|e| e.into_component::<ColliderHandle>().ok())
                    .flatten()
                {
                    physics.colliders.remove(collider_handle.0);
                    commands.remove_component::<ColliderHandle>(entity);
                }
            }
            Event::ArchetypeCreated(_arch) => {}
            _ => {}
        }
    }
}

fn entity_world_to_physics_world_system() -> impl ::legion::systems::Runnable {
    SystemBuilder::new("entity_world_to_physics_world")
        .read_component::<BodyHandle>()
        .read_component::<WorldPosition>()
        .read_component::<Velocity>()
        .read_component::<Orientation>()
        .read_component::<DynamicBody>()
        .write_resource::<PhysicsResource>()
        .build(move |cmd, world, resources, query| {
            entity_world_to_physics_world(world, &mut *resources);
        })
}

#[allow(dead_code)]
fn entity_world_to_physics_world(world: &SubWorld, physics: &mut PhysicsResource) {
    let mut query = <(
        Entity,
        Read<BodyHandle>,
        Read<WorldPosition>,
        Read<Velocity>,
        Read<Orientation>,
    )>::query()
    .filter(component::<DynamicBody>());
    for (_ent, han, pos, vel, ori) in query.iter(world) {
        if let Some(body) = physics.bodies.rigid_body_mut(han.0) {
            body.set_position(Isometry2::new(c2n(pos.0), cgmath::Rad::from(ori.0).0));
            body.set_linear_velocity(c2n(vel.0));
            // and force?
        }
    }
}

fn step_physics_world_system() -> impl ::legion::systems::Runnable {
    SystemBuilder::new("step_physics_world")
        .write_resource::<PhysicsResource>()
        .build(move |cmd, world, resources, query| {
            step_physics_world(&mut *resources);
        })
}

#[allow(dead_code)]
fn step_physics_world(physics: &mut PhysicsResource) { physics.step(); }

fn physics_world_to_entity_world_system() -> impl ::legion::systems::Runnable {
    SystemBuilder::new("physics_world_to_entity_world")
        .read_component::<BodyHandle>()
        .write_component::<WorldPosition>()
        .write_component::<Velocity>()
        .write_component::<Orientation>()
        .read_resource::<PhysicsResource>()
        .build(move |cmd, world, resources, query| {
            physics_world_to_entity_world(world, cmd, &*resources);
        })
}

#[allow(dead_code)]
fn physics_world_to_entity_world(
    world: &mut SubWorld,
    _commands: &mut CommandBuffer,
    physics: &PhysicsResource,
) {
    let mut query = <(
        Read<BodyHandle>,
        TryWrite<WorldPosition>,
        TryWrite<Velocity>,
        TryWrite<Orientation>,
    )>::query()
    .filter(component::<DynamicBody>() & maybe_changed::<BodyHandle>());
    for (body, pos, vel, ori) in query.iter_mut(world) {
        if let Some(bod) = physics.bodies.rigid_body(body.0) {
            if let Some(p) = pos {
                p.0 = n2c(bod.position().translation.vector);
            }
            if let Some(v) = vel {
                v.0 = n2c(bod.velocity().linear);
            }
            if let Some(o) = ori {
                o.0 = cgmath::Deg::from(cgmath::Rad(bod.position().rotation.angle()));
            }
        }
    }
}

fn movement_system() -> impl ::legion::systems::Runnable {
    SystemBuilder::new("movement")
        .read_resource::<FrameTime>()
        .with_query(<(::legion::Write<WorldPosition>, ::legion::Write<Velocity>)>::query())
        .build(move |cmd, world, resources, query| {
            let for_query = world;
            query.for_each_mut(for_query, |components| {
                movement(&*resources, components.0, components.1);
            });
        })
}

#[allow(dead_code)]
fn movement(frame_time: &FrameTime, pos: &mut WorldPosition, vel: &mut Velocity) {
    if vel.0.x.is_finite() && vel.0.y.is_finite() {
        let v = if (vel.0 * frame_time.0).magnitude() < 0.5 {
            vel.0 * frame_time.0
        } else {
            (vel.0 * frame_time.0).normalize() * 0.5
        };
        pos.0 += v;
    } else {
        // TODO: We need to deal with this somehow
        vel.0 = Vector2::new(0.0, 0.0);
        println!("Velocity Hickup");
    }
}
fn n2c(input: nalgebra::Vector2<f32>) -> Vector2<f32> {
    return cgmath::Vector2::new(input.x, input.y);
}
fn c2n(input: cgmath::Vector2<f32>) -> nalgebra::Vector2<f32> { return [input.x, input.y].into(); }
