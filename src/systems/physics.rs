use cgmath::{prelude::*, Vector2};

use crate::components::*;

use legion::systems::{Builder, CommandBuffer};
use legion::world::{ComponentError, EntityAccessError, EntryRef, Event, EventSender, SubWorld};
use legion::*;

use crossbeam_channel::{Receiver, Sender};

use nphysics2d::algebra::ForceType::VelocityChange;
use nphysics2d::force_generator::DefaultForceGeneratorSet;
use nphysics2d::joint::DefaultJointConstraintSet;
use nphysics2d::object::{
    BodyStatus, DefaultBodyHandle, DefaultBodySet, DefaultColliderSet, RigidBody, RigidBodyDesc,
};
use nphysics2d::world::{DefaultGeometricalWorld, DefaultMechanicalWorld};
use std::ops::Deref;

pub(crate) trait PhysicsBuilderExtender {
    fn add_physics_systems(&mut self, world: &mut World, resources: &mut Resources) -> &mut Self;
}

impl PhysicsBuilderExtender for Builder {
    fn add_physics_systems(&mut self, world: &mut World, resources: &mut Resources) -> &mut Self {
        let mut phyre = resources.get_mut_or_default::<PhysicsResource>();
        // TODO: these could be unified into a single channel
        let (sender_dynamic, receiver_dynamic) = crossbeam_channel::unbounded::<Event>();
        let (sender_static, receiver_static) = crossbeam_channel::unbounded::<Event>();
        world.subscribe(sender_dynamic, component::<DynamicBody>());
        world.subscribe(sender_static, component::<StaticBody>());
        return self
            .add_system(mirror_physics_world_system(
                receiver_dynamic,
                receiver_static,
            ))
            .add_system(pre_physics_2d_stat_dyna_check_system())
            .add_system(pre_physics_2d_dyna_vel_check_system())
            .add_system(step_physics_world_system())
            .add_system(update_from_physics_world_system())
            .add_system(movement_system());
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
fn mirror_physics_world(
    world: &mut SubWorld,
    commands: &mut legion::systems::CommandBuffer,
    #[resource] physics: &mut PhysicsResource,
    #[state] dyna_recv: &mut Receiver<Event>,
    #[state] stat_recv: &mut Receiver<Event>,
) {
    while let Ok(event) = dyna_recv.try_recv() {
        match event {
            Event::EntityInserted(entity, _) => {
                let mut body = RigidBodyDesc::<f32>::new()
                    .gravity_enabled(false)
                    .status(BodyStatus::Dynamic);
                if let Ok(entry) = world.entry_ref(entity) {
                    if let Ok(dyn_body) = entry.get_component::<DynamicBody>() {
                        body = body.mass(dyn_body.mass);
                    } else {
                        panic!("Inserting a DynamicBody without a DynamicBody!?");
                    }
                    if let Ok(pos) = entry.get_component::<Position>() {
                        body = body.translation(c2n(pos.0));
                    } else {
                        eprintln!("Inserting a DynamicBody with no Position!");
                    }
                    if let Ok(vel) = entry.get_component::<Velocity>() {
                        body = body.velocity(nphysics2d::algebra::Velocity2::new(c2n(vel.0), 0.0));
                    } else {
                        eprintln!("Inserting a DynamicBody with no Velocity!");
                    }
                    if let Ok(orient) = entry.get_component::<Orientation>() {
                        body = body.rotation(cgmath::Rad::from(orient.0).0);
                    } else {
                        eprintln!("Inserting a DynamicBody with no Orientation!");
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
    while let Ok(event) = stat_recv.try_recv() {
        match event {
            Event::EntityInserted(entity, _) => {
                let mut body = RigidBodyDesc::<f32>::new()
                    .gravity_enabled(false)
                    .status(BodyStatus::Static);
                if let Ok(e) = world.entry_ref(entity) {
                    if let Ok(p) = e.get_component::<Position>() {
                        body = body.translation(c2n(p.0));
                    } else {
                        eprintln!("Inserting a StaticBody with no Position!");
                    }
                    if let Ok(o) = e.get_component::<Orientation>() {
                        body = body.rotation(cgmath::Rad::from(o.0).0);
                    } else {
                        //eprintln!("Inserting a StaticBody with no Orientation!");
                    }
                }
                let handle = physics.bodies.insert(body.build());
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
#[read_component(Position)]
fn step_physics_world(
    world: &mut SubWorld,
    commands: &mut CommandBuffer,
    #[resource] physics: &mut PhysicsResource,
) {
    physics.step();
    // STEP HERE
}

#[system]
#[read_component(DefaultBodyHandle)]
#[write_component(Position)]
#[write_component(Velocity)]
#[write_component(Orientation)]
fn update_from_physics_world(
    world: &mut SubWorld,
    commands: &mut CommandBuffer,
    #[resource] physics: &PhysicsResource,
) {
    let mut query = <(
        Read<DefaultBodyHandle>,
        TryWrite<Position>,
        TryWrite<Velocity>,
        TryWrite<Orientation>,
    )>::query();
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

#[system(for_each)]
fn pre_physics_2d_stat_dyna_check(stat: &StaticBody, dyna: &DynamicBody) {
    panic!("There's a naughty static body that really feels dynamic inside.");
}

#[system(for_each)]
#[filter(! component::< Velocity > ())]
fn pre_physics_2d_dyna_vel_check(dyna: &DynamicBody) {
    panic!("A dynamic entity has no velocity!");
}
