use cgmath::{prelude::*, Vector2};

use crate::components::*;
use crate::Universe;

use legion::systems::Builder;
use legion::world::{ComponentError, EntityAccessError, EntryRef, Event, EventSender, SubWorld};
use legion::*;

use ncollide2d::pipeline::{CollisionObject, CollisionObjectSet};
use nphysics2d::force_generator::DefaultForceGeneratorSet;
use nphysics2d::joint::DefaultJointConstraintSet;
use nphysics2d::object::{
    Body, BodySet, Collider, ColliderRemovalData, ColliderSet, DefaultBodySet, DefaultColliderSet,
    Multibody, RigidBody,
};
use nphysics2d::world::{DefaultGeometricalWorld, DefaultMechanicalWorld};
use std::borrow::BorrowMut;

pub(crate) trait PhysicsBuilderExtender {
    fn add_physics_systems(&mut self) -> &mut Self;
}

impl PhysicsBuilderExtender for Builder {
    fn add_physics_systems(&mut self) -> &mut Self {
        self.add_system(pre_physics_2d_stat_dyna_check_system())
            .add_system(pre_physics_2d_dyna_vel_check_system())
            .add_system(init_velocity_accumulator_system())
            .add_system(physics_system())
            .add_system(velocity_update_system())
            .add_system(movement_system())
    }
}

pub struct PhysicsResource {
    mechanical_world: DefaultMechanicalWorld<f32>,
    geometrical_world: DefaultGeometricalWorld<f32>,
    bodies: DefaultBodySet<f32>,
    colliders: DefaultColliderSet<f32>,
    joint_constraints: DefaultJointConstraintSet<f32>,
    force_generators: DefaultForceGeneratorSet<f32>,
}

impl PhysicsResource {
    pub fn new() -> Self {
        return PhysicsResource {
            mechanical_world: DefaultMechanicalWorld::new(
                nalgebra::zero::<nalgebra::Vector2<f32>>(),
            ),
            geometrical_world: DefaultGeometricalWorld::new(),
            bodies: DefaultBodySet::new(),
            colliders: DefaultColliderSet::new(),
            joint_constraints: DefaultJointConstraintSet::new(),
            force_generators: DefaultForceGeneratorSet::new(),
        };
    }
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

impl nphysics2d::object::BodySet<f32> for Universe {
    type Handle = legion::Entity;

    // TODO: make these fns handle more Body types
    fn get<'a>(&self, handle: Self::Handle) -> Option<&dyn Body<f32>> {
        self.world
            .entry_ref(handle)
            .ok()
            .map(|entry| entry.into_component::<RigidBody<f32>>().ok())
            .flatten()
            .map(|comp| comp as &dyn Body<f32>)
    }

    fn get_mut(&mut self, handle: Self::Handle) -> Option<&mut dyn Body<f32>> {
        self.world
            .entry_mut(handle)
            .ok()
            .map(|entry| entry.into_component_mut::<RigidBody<f32>>().ok())
            .flatten()
            .map(|comp| comp as &mut dyn Body<f32>)
    }

    fn contains(&self, handle: Self::Handle) -> bool {
        self.world
            .entry_ref(handle)
            .ok()
            .map(|entry| entry.into_component::<RigidBody<f32>>().ok())
            .flatten()
            .is_some()
    }

    fn foreach(&self, f: &mut dyn FnMut(Self::Handle, &dyn Body<f32>)) {
        let mut query = <(Entity, &RigidBody<f32>)>::query();
        query.for_each(&self.world, |(entity, body)| {
            f(*entity, body as &dyn Body<f32>)
        });
    }

    fn foreach_mut(&mut self, f: &mut dyn FnMut(Self::Handle, &mut dyn Body<f32>)) {
        let mut query = <(Entity, &mut RigidBody<f32>)>::query();
        query.for_each_mut(&mut self.world, |(entity, body)| {
            f(*entity, body as &mut dyn Body<f32>)
        });
    }

    fn pop_removal_event(&mut self) -> Option<Self::Handle> {
        self.body_receiver
            .recv()
            .ok()
            .map(|event| match event {
                Event::EntityRemoved(entity, _) => Some(entity),
                _ => None,
            })
            .flatten()
    }
}

impl CollisionObjectSet<f32> for Universe {
    type CollisionObject = Collider<f32, Entity>;
    type CollisionObjectHandle = Entity;

    fn collision_object(
        &self,
        handle: Self::CollisionObjectHandle,
    ) -> Option<&Self::CollisionObject> {
        self.world
            .entry_ref(handle)
            .ok()
            .map(|entry| entry.into_component::<Self::CollisionObject>().ok())
            .flatten()
    }

    fn foreach(&self, mut f: impl FnMut(Self::CollisionObjectHandle, &Self::CollisionObject)) {
        let mut query = <(Entity, &Self::CollisionObject)>::query();
        query.for_each(&self.world, |(entity, collision_object)| {
            f(*entity, collision_object);
        });
    }
}

impl nphysics2d::object::ColliderSet<f32, Entity> for Universe {
    type Handle = Entity;

    fn get(&self, handle: Self::Handle) -> Option<&Collider<f32, Entity>> {
        self.world
            .entry_ref(handle)
            .ok()
            .map(|entry| entry.into_component::<Collider<f32, Entity>>().ok())
            .flatten()
    }

    fn get_mut(&mut self, handle: Self::Handle) -> Option<&mut Collider<f32, Entity>> {
        self.world
            .entry_mut(handle)
            .ok()
            .map(|entry| entry.into_component_mut::<Collider<f32, Entity>>().ok())
            .flatten()
    }

    fn contains(&self, handle: Self::Handle) -> bool {
        self.world
            .entry_ref(handle)
            .ok()
            .map(|entry| entry.into_component::<Collider<f32, Entity>>().ok())
            .flatten()
            .is_some()
    }

    fn foreach(&self, mut f: impl FnMut(Self::Handle, &Collider<f32, Entity>)) {
        let mut query = <(Entity, &Collider<f32, Entity>)>::query();
        query.for_each(&self.world, |(entity, collider)| f(*entity, collider));
    }

    fn foreach_mut(&mut self, mut f: impl FnMut(Self::Handle, &mut Collider<f32, Entity>)) {
        let mut query = <(Entity, &mut Collider<f32, Entity>)>::query();
        query.for_each_mut(&mut self.world, |(entity, collider)| f(*entity, collider));
    }

    fn pop_insertion_event(&mut self) -> Option<Self::Handle> {
        unimplemented!()
    }

    fn pop_removal_event(&mut self) -> Option<(Self::Handle, ColliderRemovalData<f32, Entity>)> {
        unimplemented!()
    }

    fn remove(&mut self, to_remove: Self::Handle) -> Option<&mut ColliderRemovalData<f32, Entity>> {
        let result = self
            .world
            .entry_mut(to_remove)
            .ok()
            .map(|enty| {
                enty.into_component::<Collider<f32, Entity>>()
                    .ok()
                    .map(|collider| collider.removal_data())
                    .flatten()
            })
            .flatten()
            .map(|rd| {
                self.removed.push((to_remove, rd));
                self.removed.last_mut().map(|r| &mut r.1)
            });
        self.world.remove(to_remove);
        return result;
    }
}

#[system(for_each)]
pub fn movement(#[resource] frame_time: &FrameTime, pos: &mut Position, vel: &mut Velocity) {
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
pub fn pre_physics_2d_stat_dyna_check(stat: &StaticBody, dyna: &DynamicBody) {
    panic!("There's a naughty static body that really feels dynamic inside.");
}

#[system(for_each)]
#[filter(! component::< Velocity > ())]
pub fn pre_physics_2d_dyna_vel_check(dyna: &DynamicBody) {
    panic!("A dynamic entity has no velocity!");
}

#[system(for_each)]
pub fn damping(dyna: &DynamicBody, vel: &mut Velocity) {
    let damping = 4.0;
    // vel.0 *= 1. - (damping * frame_time.0).min(1.);
}

#[system]
fn physics() {}

#[system(for_each)]
pub fn init_velocity_accumulator(vel: &Velocity, velacc: &mut VelocityAccumulator) {
    *velacc = VelocityAccumulator::zero();
}

#[system(for_each)]
pub fn velocity_update(vel: &mut Velocity, velacc: &VelocityAccumulator) {
    vel.0 = velacc.get();
}
