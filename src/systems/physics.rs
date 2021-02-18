use cgmath::{prelude::*, Vector2};
use crossbeam_channel::Receiver;
use legion::query::Query;
use legion::systems::{Builder, CommandBuffer, Runnable};
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
            // TODO: reimplement .add_system(validate_physics_entities_system())
            .add_system(make_body_handles_system())
            .add_system(remove_body_handles())
            // .add_system(flush_command_buffer_system())
            .add_system(make_collider_handles_system())
            .add_system(remove_collider_handles())
            //.add_system(handle_entity_removal_system(receiver))
            // .add_system(flush_command_buffer_system())
            .add_system(entity_world_to_physics_world())
            .add_system(step_physics_world())
            .add_system(physics_world_to_entity_world());
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

fn make_body_handles_system() -> impl Runnable {
    SystemBuilder::new("make_body_handles")
        .write_resource::<PhysicsResource>()
        .with_query(
            <(Entity, Read<PhysicsBody>, TryRead<WorldPosition>)>::query()
                .filter(!component::<BodyHandle>()),
        )
        .build(move |commands, world, resources, query| {
            let (mut for_query, mut world) = world.split_for_query(query);
            let physics: &mut PhysicsResource = &mut *resources;
            query.for_each_mut(
                &mut for_query,
                |(entity, physics_body, position): (
                    &Entity,
                    &PhysicsBody,
                    Option<&WorldPosition>,
                )| {
                    let body = match physics_body {
                        PhysicsBody::Disabled => {
                            RigidBodyDesc::<f32>::new().status(BodyStatus::Disabled)
                        }
                        PhysicsBody::Static => RigidBodyDesc::<f32>::new()
                            .status(BodyStatus::Static)
                            .position(Isometry2::new(
                                c2n(position.unwrap_or(&WorldPosition(cgmath::vec2(0., 0.))).0),
                                0.,
                            )),
                        PhysicsBody::Dynamic { mass } => RigidBodyDesc::<f32>::new()
                            .status(BodyStatus::Dynamic)
                            .gravity_enabled(false)
                            .mass(*mass),
                    };
                    let handle = BodyHandle(physics.bodies.insert(body.build()));
                    commands.add_component(*entity, handle);
                },
            );
        })
}

fn remove_body_handles() -> impl Runnable {
    SystemBuilder::new("remove_body_handles")
        .write_resource::<PhysicsResource>()
        .with_query(<(Entity, Read<BodyHandle>)>::query().filter(!component::<PhysicsBody>()))
        .build(move |commands, world, physics, query| {
            query.for_each_mut(world, |(entity, handle): (&Entity, &BodyHandle)| {
                physics.bodies.remove(handle.0);
                commands.remove_component::<BodyHandle>(*entity);
            });
        })
}

fn flush_command_buffer_system() -> impl Runnable {
    SystemBuilder::new("flush_command_buffer")
        .with_query(<(Write<World>)>::query())
        .build(move |commands, world, _, query| {
            let for_query = world; // TODO: simplify this
            query.for_each_mut(for_query, |components| {
                commands.flush(components);
            });
        })
}

fn make_collider_handles_system() -> impl Runnable {
    SystemBuilder::new("make_collider_handles")
        .write_resource::<PhysicsResource>()
        .with_query(
            <(Entity, Read<BodyHandle>, Read<Collider>)>::query()
                .filter(!component::<ColliderHandle>()),
        )
        .build(move |cmd, world, resources, query| {
            let (mut for_query, world) = world.split_for_query(query);
            let for_query = &mut for_query;
            query.for_each_mut(for_query, |components| {
                make_collider_handles(
                    cmd,
                    &mut *resources,
                    components.0,
                    components.1,
                    components.2,
                );
            });
        })
}

#[allow(dead_code)]
fn make_collider_handles(
    commands: &mut CommandBuffer,
    physics: &mut PhysicsResource,
    entity: &Entity,
    body_handle: &BodyHandle,
    collider: &Collider,
) {
    let shape_handle = match collider {
        Collider::Circle { radius } => ShapeHandle::new(Ball::new(*radius)),
        Collider::Square { side_length } => {
            let half_side = side_length / 2.0;
            let sides_vec = nalgebra::Vector2::new(half_side, half_side);
            ShapeHandle::new(Cuboid::new(sides_vec))
        }
    };
    let collider = ColliderDesc::<f32>::new(shape_handle);
    let handle = ColliderHandle(
        physics
            .colliders
            .insert(collider.build(BodyPartHandle(body_handle.0, 0))),
    );
    commands.add_component(*entity, handle);
}

fn remove_collider_handles() -> impl Runnable {
    SystemBuilder::new("remove_collider_handles")
        .write_resource::<PhysicsResource>()
        .with_query(<(Entity, Read<ColliderHandle>)>::query().filter(!component::<Collider>()))
        .build(move |commands, world, physics, query| {
            let for_query = world;
            query.for_each_mut(for_query, |(entity, collider_handle)| {
                physics.colliders.remove(collider_handle.0);
                commands.remove_component::<ColliderHandle>(*entity);
            });
        })
}

fn handle_entity_removal_system(state_0: Receiver<Event>) -> impl Runnable {
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
    commands: &mut CommandBuffer,
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

fn entity_world_to_physics_world() -> impl Runnable {
    SystemBuilder::new("entity_world_to_physics_world")
        .read_component::<BodyHandle>()
        .read_component::<WorldPosition>()
        .read_component::<Velocity>()
        .read_component::<Orientation>()
        .read_component::<PhysicsBody>()
        .write_resource::<PhysicsResource>()
        .with_query(<(
            Read<BodyHandle>,
            Read<PhysicsBody>,
            Read<WorldPosition>,
            Read<Velocity>,
            Read<Orientation>,
        )>::query())
        .build(move |_, world, physics, query| {
            let physics: &mut PhysicsResource = &mut *physics;
            for (han, bod, pos, vel, ori) in query.iter(world) {
                if let PhysicsBody::Dynamic { .. } = bod {
                    if let Some(body) = physics.bodies.rigid_body_mut(han.0) {
                        body.set_position(Isometry2::new(c2n(pos.0), cgmath::Rad::from(ori.0).0));
                        body.set_linear_velocity(c2n(vel.0));
                        // and force?
                    }
                }
            }
        })
}

fn step_physics_world() -> impl Runnable {
    SystemBuilder::new("step_physics_world")
        .write_resource::<PhysicsResource>()
        .build(move |_, _, physics, _| {
            let physics: &mut PhysicsResource = &mut *physics;
            physics.step();
        })
}

fn physics_world_to_entity_world() -> impl Runnable {
    SystemBuilder::new("physics_world_to_entity_world")
        .read_component::<BodyHandle>()
        .read_component::<PhysicsBody>()
        .write_component::<WorldPosition>()
        .write_component::<Velocity>()
        .write_component::<Orientation>()
        .read_resource::<PhysicsResource>()
        .build(move |_, world, resources, query| {
            let physics: &PhysicsResource = &*resources;
            let mut query = <(
                Read<BodyHandle>,
                Read<PhysicsBody>,
                TryWrite<WorldPosition>,
                TryWrite<Velocity>,
                TryWrite<Orientation>,
            )>::query()
            .filter(maybe_changed::<BodyHandle>());
            for (handle, body, pos, vel, ori) in query.iter_mut(world) {
                if let PhysicsBody::Dynamic { .. } = body {
                    if let Some(bod) = physics.bodies.rigid_body(handle.0) {
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
        })
}

fn movement_system() -> impl Runnable {
    SystemBuilder::new("movement")
        .read_resource::<FrameTime>()
        .with_query(<(Write<WorldPosition>, Write<Velocity>)>::query())
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
