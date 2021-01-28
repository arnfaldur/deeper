use cgmath::{prelude::*, Vector2};

use crate::components::*;

use legion::systems::{Builder, CommandBuffer};
use legion::world::{ComponentError, EntityAccessError, EntryRef, Event, EventSender, SubWorld};
use legion::*;

use crossbeam_channel::{Receiver, Sender};

use nalgebra::Isometry2;
use nphysics2d::algebra::Force2;
use nphysics2d::algebra::ForceType::VelocityChange;
use nphysics2d::force_generator::DefaultForceGeneratorSet;
use nphysics2d::joint::DefaultJointConstraintSet;
use nphysics2d::object::{
    Body, BodyStatus, DefaultBodyHandle, DefaultBodySet, DefaultColliderSet, RigidBody,
    RigidBodyDesc,
};
use nphysics2d::world::{DefaultGeometricalWorld, DefaultMechanicalWorld};

pub(crate) trait PhysicsBuilderExtender {
    fn add_physics_systems(&mut self, world: &mut World, resources: &mut Resources) -> &mut Self;
}

impl PhysicsBuilderExtender for Builder {
    fn add_physics_systems(&mut self, world: &mut World, resources: &mut Resources) -> &mut Self {
        let mut phyre = resources.get_mut_or_default::<PhysicsResource>();
        let (sender, receiver) = crossbeam_channel::unbounded::<Event>();
        world.subscribe(
            sender,
            component::<DynamicBody>() | component::<StaticBody>(),
        );
        return self
            .add_system(validate_physics_entities_system())
            .add_system(entity_samsara_system(receiver))
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

fn c2n(input: cgmath::Vector2<f32>) -> nalgebra::Vector2<f32> {
    return [input.x, input.y].into();
}

#[system]
#[read_component(Position)]
#[read_component(Velocity)]
#[read_component(Orientation)]
#[read_component(DynamicBody)]
#[read_component(StaticBody)]
fn entity_samsara(
    // Samsara = the cycle of birth and death
    world: &mut SubWorld,
    commands: &mut legion::systems::CommandBuffer,
    #[resource] physics: &mut PhysicsResource,
    #[state] receiver: &mut Receiver<Event>,
) {
    while let Ok(event) = receiver.try_recv() {
        match event {
            Event::EntityInserted(entity, _) => {
                let mut body = RigidBodyDesc::<f32>::new().gravity_enabled(false);
                if let Ok(entry) = world.entry_ref(entity) {
                    if let Ok(dyn_body) = entry.get_component::<DynamicBody>() {
                        body = body.mass(dyn_body.mass).status(BodyStatus::Dynamic);
                        if let Ok(spd) = entry.get_component::<Speed>() {
                            body = body.max_linear_velocity(spd.0);
                        }
                        if let Ok(pos) = entry.get_component::<Position>() {
                            body = body.translation(c2n(pos.0));
                        }
                        if let Ok(vel) = entry.get_component::<Velocity>() {
                            body =
                                body.velocity(nphysics2d::algebra::Velocity2::new(c2n(vel.0), 0.0));
                        }
                        if let Ok(orient) = entry.get_component::<Orientation>() {
                            body = body.rotation(cgmath::Rad::from(orient.0).0);
                        }
                    }
                    if let Ok(_) = entry.get_component::<StaticBody>() {
                        if let Ok(p) = entry.get_component::<Position>() {
                            body = body.translation(c2n(p.0));
                        }
                        if let Ok(o) = entry.get_component::<Orientation>() {
                            body = body.rotation(cgmath::Rad::from(o.0).0);
                        }
                    }
                }
                let body = body.build();
                let handle = physics.bodies.insert(body);
                commands.add_component(entity, handle);
            }
            Event::EntityRemoved(entity, _) => {
                if let Ok(e) = world.entry_mut(entity) {
                    if let Ok(b) = e.get_component::<DefaultBodyHandle>() {
                        physics.bodies.remove(*b);
                    }
                }
            }
            _ => {}
        }
    }
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
        }
        if vel.is_none() {
            panic!("missing Velocity in DynamicBody");
        }
        // TODO: decide if Force should be in our repertoire
        // if frc.is_none() {
        //     commands.add_component(*ent, Force::default());
        // }
        if ori.is_none() {
            panic!("missing Orientation in DynamicBody");
        }
        if sta.is_some() {
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
        }
        if vel.is_some() {
            panic!("StaticBody can't have a Velocity component");
        }
        if spd.is_some() {
            panic!("StaticBody can't have a Speed component");
        }
        if acc.is_some() {
            panic!("StaticBody can't have a Acceleration component");
        }
        if frc.is_some() {
            panic!("StaticBody can't have a Force component");
        }
        // if ori.is_none() {
        //     panic!("missing Orientation in StaticBody");
        // }
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
                // TODO: check if this is deg or rad
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
