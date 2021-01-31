use cgmath::{prelude::*, Vector2};

use legion::systems::{Builder, CommandBuffer};
use legion::world::{ComponentError, EntityAccessError, EntryRef, Event, EventSender, SubWorld};
use legion::*;

use crossbeam_channel::Receiver;

use nalgebra::Isometry2;

use nphysics2d::force_generator::DefaultForceGeneratorSet;
use nphysics2d::joint::DefaultJointConstraintSet;
use nphysics2d::object::{
    Body, BodyPartHandle, BodyStatus, ColliderDesc, DefaultBodyHandle, DefaultBodySet,
    DefaultColliderHandle, DefaultColliderSet, RigidBodyDesc,
};
use nphysics2d::world::{DefaultGeometricalWorld, DefaultMechanicalWorld};

use crate::components::*;
use legion::storage::ArchetypeIndex;
use ncollide2d::shape::ShapeHandle;
use nphysics2d::ncollide2d::shape::{Ball, Cuboid};

pub(crate) trait PhysicsBuilderExtender {
    fn add_physics_systems(&mut self, world: &mut World, resources: &mut Resources) -> &mut Self;
}

impl PhysicsBuilderExtender for Builder {
    fn add_physics_systems(&mut self, world: &mut World, resources: &mut Resources) -> &mut Self {
        let phyre = resources.get_mut_or_default::<PhysicsResource>();
        let (sender, receiver) = crossbeam_channel::unbounded::<Event>();
        let (collider_sender, collider_receiver) = crossbeam_channel::unbounded::<Event>();
        world.subscribe(
            sender,
            component::<DynamicBody>()
                | component::<StaticBody>()
                | component::<DisabledBody>()
                | component::<CircleCollider>()
                | component::<SquareCollider>(),
        );
        return self
            .add_system(validate_physics_entities_system())
            .add_system(samsara_system(receiver))
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

fn n2c(input: nalgebra::Vector2<f32>) -> Vector2<f32> {
    return cgmath::Vector2::new(input.x, input.y);
}

fn c2n(input: cgmath::Vector2<f32>) -> nalgebra::Vector2<f32> { return [input.x, input.y].into(); }

#[system]
#[read_component(Position)]
#[read_component(Velocity)]
#[read_component(Speed)]
#[read_component(Acceleration)]
#[write_component(Force)]
#[read_component(Orientation)]
#[read_component(DynamicBody)]
#[read_component(StaticBody)]
fn validate_physics_entities(world: &mut SubWorld, commands: &mut CommandBuffer) {
    let mut query = <(
        Entity,
        TryRead<Position>,
        TryRead<Velocity>,
        TryRead<Force>,
        TryRead<Orientation>,
        TryRead<StaticBody>,
    )>::query()
    .filter(component::<DynamicBody>());
    for (ent, pos, vel, frc, ori, sta) in query.iter(world) {
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
        TryRead<Position>,
        TryRead<Velocity>,
        TryRead<Speed>,
        TryRead<Acceleration>,
        TryRead<Force>,
        TryRead<Orientation>,
    )>::query()
    .filter(component::<StaticBody>());
    for (ent, pos, vel, spd, acc, frc, ori) in query.iter(world) {
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

#[system(for_each)]
#[filter(!component::<DefaultBodyHandle>())]
fn make_dynamic_body_handles(
    commands: &mut legion::systems::CommandBuffer,
    #[resource] physics: &mut PhysicsResource,
    entity: Entity,
    dynamic: &DynamicBody,
) {
    let body = RigidBodyDesc::<f32>::new()
        .gravity_enabled(false)
        .mass(dynamic.mass)
        .status(BodyStatus::Dynamic)
        .build();
    let handle = physics.bodies.insert(body);
    commands.add_component(entity, handle);
}

#[system(for_each)]
#[filter(component::<StaticBody> & !component::<DefaultBodyHandle>())]
fn make_static_body_handles(
    commands: &mut legion::systems::CommandBuffer,
    #[resource] physics: &mut PhysicsResource,
    entity: Entity,
    stat: &StaticBody,
) {
    let body = RigidBodyDesc::<f32>::new()
        .status(BodyStatus::Static)
        .build();
    let handle = physics.bodies.insert(body);
    commands.add_component(entity, handle);
}

#[system(for_each)]
#[filter(component::<DisabledBody> & !component::<DefaultBodyHandle>())]
fn make_disabled_body_handles(
    commands: &mut legion::systems::CommandBuffer,
    #[resource] physics: &mut PhysicsResource,
    entity: Entity,
) {
    let body = RigidBodyDesc::<f32>::new()
        .status(BodyStatus::Disabled)
        .build();
    let handle = physics.bodies.insert(body);
    commands.add_component(entity, handle);
}

#[system]
#[read_component(Position)]
#[read_component(Velocity)]
#[read_component(Orientation)]
#[read_component(DynamicBody)]
#[read_component(StaticBody)]
fn samsara(
    // Samsara = the cycle of birth and death
    world: &mut SubWorld,
    commands: &mut legion::systems::CommandBuffer,
    #[resource] physics: &mut PhysicsResource,
    #[state] receiver: &mut Receiver<Event>,
) {
    while let Ok(event) = receiver.try_recv() {
        match event {
            Event::EntityInserted(entity, a) => {
                if let Ok(entry) = world.entry_ref(entity) {
                    let body_handle = match entry.get_component::<DefaultBodyHandle>() {
                        Ok(handle) => Some(*handle),
                        Err(_) => {
                            let body_description =
                                if let Ok(dyn_body) = entry.get_component::<DynamicBody>() {
                                    // println!("dynamic body");
                                    Some(get_dyn_body_desc(&entry, dyn_body))
                                } else if let Ok(_) = entry.get_component::<StaticBody>() {
                                    // println!("static body");
                                    Some(get_static_body_desc(&entry))
                                } else if let Ok(_) = entry.get_component::<DisabledBody>() {
                                    println!("disabled body");
                                    Some(RigidBodyDesc::new().status(BodyStatus::Disabled))
                                } else {
                                    println!("nobody");
                                    None
                                };

                            if let Some(body) = body_description {
                                let body = body.build();
                                let handle = physics.bodies.insert(body);
                                commands.add_component(entity, handle);
                                // this is kind of a return statement:
                                Some(handle)
                            } else {
                                None
                            }
                        }
                    };
                    if let Some(bhandle) = body_handle {
                        let shape_handle = if let Ok(circle_collider) =
                            entry.get_component::<CircleCollider>()
                        {
                            println!("circle collider");
                            Some(ShapeHandle::new(Ball::new(circle_collider.radius)))
                        } else if let Ok(square_collider) = entry.get_component::<SquareCollider>()
                        {
                            println!("square collider");
                            let side_length = square_collider.side_length / 2.0;
                            let sides_vec = nalgebra::Vector2::new(side_length, side_length);
                            Some(ShapeHandle::new(Cuboid::new(sides_vec)))
                        } else {
                            // println!("no collider");
                            None
                        };
                        if let Some(shape) = shape_handle {
                            let collider = ColliderDesc::new(shape);
                            let collider = collider.build(BodyPartHandle(bhandle, 0));
                            let handle = physics.colliders.insert(collider);
                            commands.add_component(entity, handle);
                            println!("adding collider ;)");
                        }
                    }
                }
            }
            Event::EntityRemoved(entity, _) => {
                if let Ok(e) = world.entry_mut(entity) {
                    if let Ok(b) = e.get_component::<DefaultBodyHandle>() {
                        physics.bodies.remove(*b);
                    }
                    if let Ok(c) = e.get_component::<DefaultColliderHandle>() {
                        physics.colliders.remove(*c);
                    }
                }
            }
            _ => {}
        }
    }
}

fn get_dyn_body_desc(entry: &EntryRef, dyn_body: &DynamicBody) -> RigidBodyDesc<f32> {
    let mut body = RigidBodyDesc::<f32>::new()
        .gravity_enabled(false)
        .mass(dyn_body.mass)
        .status(BodyStatus::Dynamic);
    if let Ok(spd) = entry.get_component::<Speed>() {
        body = body.max_linear_velocity(spd.0);
    }
    if let Ok(pos) = entry.get_component::<Position>() {
        body = body.translation(c2n(pos.0));
    }
    if let Ok(vel) = entry.get_component::<Velocity>() {
        body = body.velocity(nphysics2d::algebra::Velocity2::new(c2n(vel.0), 0.0));
    }
    if let Ok(orient) = entry.get_component::<Orientation>() {
        body = body.rotation(cgmath::Rad::from(orient.0).0);
    }
    return body;
}

fn get_static_body_desc(entry: &EntryRef) -> RigidBodyDesc<f32> {
    let mut body = RigidBodyDesc::<f32>::new()
        .gravity_enabled(false)
        .status(BodyStatus::Static);
    if let Ok(p) = entry.get_component::<Position>() {
        body = body.translation(c2n(p.0));
    }
    if let Ok(o) = entry.get_component::<Orientation>() {
        body = body.rotation(cgmath::Rad::from(o.0).0);
    }
    return body;
}

#[system]
#[read_component(DefaultBodyHandle)]
#[read_component(Position)]
#[read_component(Velocity)]
#[read_component(Orientation)]
#[read_component(DynamicBody)]
fn entity_world_to_physics_world(
    world: &mut SubWorld,
    commands: &mut legion::systems::CommandBuffer,
    #[resource] physics: &mut PhysicsResource,
) {
    let mut query = <(
        Entity,
        Read<DefaultBodyHandle>,
        Read<Position>,
        Read<Velocity>,
        Read<Orientation>,
    )>::query()
    .filter(component::<DynamicBody>());
    for (ent, han, pos, vel, ori) in query.iter(world) {
        if let Some(body) = physics.bodies.rigid_body_mut(*han) {
            body.set_position(Isometry2::new(c2n(pos.0), cgmath::Rad::from(ori.0).0));
            body.set_linear_velocity(c2n(vel.0));
            // and force?
        }
    }
}

#[system]
#[read_component(Position)]
fn step_physics_world(
    world: &mut SubWorld,
    commands: &mut CommandBuffer,
    #[resource] physics: &mut PhysicsResource,
) {
    physics.step();
}

#[system]
#[read_component(DefaultBodyHandle)]
#[write_component(Position)]
#[write_component(Velocity)]
#[write_component(Orientation)]
fn physics_world_to_entity_world(
    world: &mut SubWorld,
    commands: &mut CommandBuffer,
    #[resource] physics: &PhysicsResource,
) {
    let mut query = <(
        Read<DefaultBodyHandle>,
        TryWrite<Position>,
        TryWrite<Velocity>,
        TryWrite<Orientation>,
    )>::query()
    .filter(component::<DynamicBody>());
    for (body, pos, vel, ori) in query.iter_mut(world) {
        physics.bodies.rigid_body(*body).map(|bod| {
            if let Some(p) = pos {
                p.0 = n2c(bod.position().translation.vector);
            }
            if let Some(v) = vel {
                v.0 = n2c(bod.velocity().linear);
            }
            if let Some(o) = ori {
                o.0 = cgmath::Deg::from(cgmath::Rad(bod.position().rotation.angle()));
            }
        });
    }
}

#[system(for_each)]
fn movement(#[resource] frame_time: &FrameTime, pos: &mut Position, vel: &mut Velocity) {
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
