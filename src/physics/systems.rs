#![allow(dead_code)]

use cgmath::{InnerSpace, Rotation3};
use crossbeam_channel::Receiver;
use legion::storage::Component;
use legion::systems::{Builder, ParallelRunnable};
use legion::world::Event;
use legion::{component, Entity, EntityStore, IntoQuery, Resources, SystemBuilder, World};
use ncollide2d::shape::ShapeHandle;
use nphysics2d::force_generator::DefaultForceGeneratorSet;
use nphysics2d::joint::DefaultJointConstraintSet;
use nphysics2d::ncollide2d::shape::{Ball, Cuboid};
use nphysics2d::object::{
    BodyPartHandle, BodyStatus, ColliderDesc, DefaultBodySet, DefaultColliderSet, RigidBodyDesc,
};
use nphysics2d::world::{DefaultGeometricalWorld, DefaultMechanicalWorld};

use crate::components::FrameTime;
use crate::physics::*;
use crate::transform::{Position, Rotation};

pub(crate) trait PhysicsBuilderExtender {
    fn add_physics_systems(&mut self, world: &mut World, resources: &mut Resources) -> &mut Self;
}

impl PhysicsBuilderExtender for Builder {
    fn add_physics_systems(&mut self, world: &mut World, resources: &mut Resources) -> &mut Self {
        resources.insert(PhysicsResource::default());
        let (sender_body, _receiver_body) = crossbeam_channel::unbounded::<Event>();
        let (sender_collider, _receiver_collider) = crossbeam_channel::unbounded::<Event>();
        world.subscribe(sender_body, component::<BodyHandle>());
        world.subscribe(sender_collider, component::<ColliderHandle>());
        return self
            // TODO: reimplement .add_system(validate_physics_entities_system())
            .add_system(make_body_handles())
            .add_system(remove_body_handles())
            .flush()
            .add_system(make_collider_handles())
            .add_system(remove_collider_handles())
            .flush()
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

// This is mostly just a proof of concept for subscribers, I left it here for future reference
fn removal_subscriber<T: Component>(receiver: Receiver<Event>) -> impl ParallelRunnable {
    SystemBuilder::new("subscription_tester")
        .read_component::<T>()
        .build(move |_, world, _, _| {
            while let Ok(boi) = receiver.try_recv() {
                if let Event::EntityRemoved(ent, _) = boi {
                    if world
                        .entry_ref(ent)
                        .map_or(true, |ent| ent.get_component::<T>().is_err())
                    {
                        println!("{:?} has been removed", ent);
                    }
                }
            }
        })
}

fn make_body_handles() -> impl ParallelRunnable {
    SystemBuilder::new("make_body_handles")
        .read_component::<PhysicsBody>()
        .read_component::<Position>()
        .write_resource::<PhysicsResource>()
        .with_query(<(Entity, &PhysicsBody, &Position)>::query().filter(!component::<BodyHandle>()))
        .build(move |commands, world, resources, query| {
            let physics: &mut PhysicsResource = &mut *resources;
            for (entity, physics_body, position) in query.iter_mut(world) {
                let body = match physics_body {
                    PhysicsBody::Disabled => {
                        RigidBodyDesc::<f32>::new().status(BodyStatus::Disabled)
                    }
                    PhysicsBody::Static => RigidBodyDesc::<f32>::new()
                        .status(BodyStatus::Static)
                        .position(nalgebra::Isometry2::new(c2n(position.0.truncate()), 0.)),
                    PhysicsBody::Dynamic { mass } => RigidBodyDesc::<f32>::new()
                        .status(BodyStatus::Dynamic)
                        .gravity_enabled(false)
                        .mass(*mass),
                };
                let handle = BodyHandle(physics.bodies.insert(body.build()));
                commands.add_component(*entity, handle);
            }
        })
}

fn remove_body_handles() -> impl ParallelRunnable {
    SystemBuilder::new("remove_body_handles")
        .read_component::<PhysicsBody>()
        .write_resource::<PhysicsResource>()
        .with_query(<(Entity, &BodyHandle)>::query().filter(!component::<PhysicsBody>()))
        .build(move |commands, world, physics, query| {
            query.for_each_mut(world, |(entity, handle): (&Entity, &BodyHandle)| {
                physics.bodies.remove(handle.0);
                commands.remove_component::<BodyHandle>(*entity);
            });
        })
}

fn make_collider_handles() -> impl ParallelRunnable {
    SystemBuilder::new("make_collider_handles")
        .read_component::<BodyHandle>()
        .read_component::<Collider>()
        .write_resource::<PhysicsResource>()
        .with_query(
            <(Entity, &BodyHandle, &Collider)>::query().filter(!component::<ColliderHandle>()),
        )
        .build(move |commands, world, resources, query| {
            // TODO: figure out if this split does anything
            // or if `world` is already the same as `for_query`
            let (mut for_query, _) = world.split_for_query(query);
            let physics: &mut PhysicsResource = &mut *resources;
            for components in query.iter_mut(&mut for_query) {
                let (entity, body_handle, collider): (&Entity, &BodyHandle, &Collider) = components;
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
        })
}

fn remove_collider_handles() -> impl ParallelRunnable {
    SystemBuilder::new("remove_collider_handles")
        .write_resource::<PhysicsResource>()
        .with_query(<(Entity, &ColliderHandle)>::query().filter(!component::<Collider>()))
        .build(move |commands, world, physics, query| {
            let for_query = world;
            query.for_each_mut(for_query, |(entity, collider_handle)| {
                physics.colliders.remove(collider_handle.0);
                commands.remove_component::<ColliderHandle>(*entity);
            });
        })
}

fn entity_world_to_physics_world() -> impl ParallelRunnable {
    SystemBuilder::new("entity_world_to_physics_world")
        .read_component::<BodyHandle>()
        .read_component::<Position>()
        .read_component::<Velocity>()
        .read_component::<Rotation>()
        .read_component::<PhysicsBody>()
        .write_resource::<PhysicsResource>()
        .with_query(<(&BodyHandle, &PhysicsBody, &Position, &Velocity, &Rotation)>::query())
        .build(move |_, world, physics, query| {
            let physics: &mut PhysicsResource = &mut *physics;
            for (han, bod, pos, vel, ori) in query.iter(world) {
                if let PhysicsBody::Dynamic { .. } = bod {
                    if let Some(body) = physics.bodies.rigid_body_mut(han.0) {
                        body.set_position(nalgebra::Isometry2::new(
                            c2n(pos.0.truncate()),
                            ori.to_rad().0,
                        ));
                        body.set_linear_velocity(c2n(vel.0));
                        // and force?
                    }
                }
            }
        })
}

fn step_physics_world() -> impl ParallelRunnable {
    SystemBuilder::new("step_physics_world")
        .read_resource::<FrameTime>()
        .write_resource::<PhysicsResource>()
        .build(move |_, _, (frame_time, physics), _| {
            let physics: &mut PhysicsResource = &mut *physics;
            physics.mechanical_world.set_timestep(frame_time.0);
            physics.step();
        })
}

fn physics_world_to_entity_world() -> impl ParallelRunnable {
    SystemBuilder::new("physics_world_to_entity_world")
        .read_component::<BodyHandle>()
        .read_component::<PhysicsBody>()
        .write_component::<Position>()
        .write_component::<Velocity>()
        .write_component::<Rotation>()
        .read_resource::<PhysicsResource>()
        .with_query(<(
            &BodyHandle,
            &PhysicsBody,
            &mut Position,
            Option<&mut Velocity>,
            Option<&mut Rotation>,
        )>::query())
        .build(move |_, world, resources, query| {
            let physics: &PhysicsResource = &*resources;
            query.for_each_mut(
                world,
                |(handle, body, pos, vel, ori): (
                    &BodyHandle,
                    &PhysicsBody,
                    &mut Position,
                    Option<&mut Velocity>,
                    Option<&mut Rotation>,
                )| {
                    if let PhysicsBody::Dynamic { .. } = body {
                        if let Some(bod) = physics.bodies.rigid_body(handle.0) {
                            pos.0 = n2c(&bod.position().translation.vector).extend(0.);
                            if let Some(v) = vel {
                                v.0 = n2c(&bod.velocity().linear);
                            }
                            if let Some(o) = ori {
                                o.0 = cgmath::Quaternion::from_angle_z(cgmath::Rad(
                                    bod.position().rotation.angle(),
                                ));
                            }
                        }
                    }
                },
            );
        })
}

fn movement_system() -> impl ParallelRunnable {
    SystemBuilder::new("movement")
        .read_resource::<FrameTime>()
        .with_query(<(&mut Position, &mut Velocity)>::query())
        .build(move |_cmd, world, resources, query| {
            let for_query = world;
            query.for_each_mut(for_query, |components| {
                movement(&*resources, components.0, components.1);
            });
        })
}

fn movement(frame_time: &FrameTime, pos: &mut Position, vel: &mut Velocity) {
    if vel.0.x.is_finite() && vel.0.y.is_finite() {
        let v = if (vel.0 * frame_time.0).magnitude() < 0.5 {
            vel.0 * frame_time.0
        } else {
            (vel.0 * frame_time.0).normalize() * 0.5
        };
        pos.0 += v.extend(0.);
    } else {
        // TODO: We need to deal with this somehow
        vel.0 = cgmath::Vector2::new(0.0, 0.0);
        println!("Velocity Hickup");
    }
}

fn n2c(input: &nalgebra::Vector2<f32>) -> cgmath::Vector2<f32> {
    return cgmath::Vector2::new(input.x, input.y);
}

fn c2n(input: cgmath::Vector2<f32>) -> nalgebra::Vector2<f32> { return [input.x, input.y].into(); }
