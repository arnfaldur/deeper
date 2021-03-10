use std::f32::consts::FRAC_PI_2;

use cgmath::{InnerSpace, Vector2, Vector3};
use legion::systems::{CommandBuffer, ParallelRunnable};
use legion::world::SubWorld;
use legion::{IntoQuery, *};

use crate::components::*;
use crate::physics::Velocity;
use crate::transform::components::{Position, Rotation};

pub mod assets;
pub mod player;
pub mod rendering;
pub mod world_gen;

#[allow(dead_code)]
pub(crate) fn order_tester(message: &'static str) -> impl ParallelRunnable {
    SystemBuilder::new("order_tester: \"".to_owned() + message + "\"").build(move |_, _, _, _| {
        eprintln!("{}", message);
    })
}

pub fn spherical_offset_system() -> impl ParallelRunnable {
    SystemBuilder::new("spherical_offset")
        .with_query(<(&mut Position, &SphericalOffset)>::query())
        .build(move |_cmd, world, _resources, query| {
            let for_query = world;
            query.for_each_mut(for_query, |components| {
                spherical_offset(components.0, components.1);
            });
        })
}

#[allow(dead_code)]
pub fn spherical_offset(pos: &mut Position, follow: &SphericalOffset) {
    pos.0.x = follow.radius * follow.theta.cos() * follow.phi.cos();
    pos.0.y = follow.radius * follow.theta.sin() * follow.phi.cos();
    pos.0.z = follow.radius * follow.phi.sin();
}

#[allow(dead_code)]
pub fn hit_point_regen_system() -> impl ParallelRunnable {
    SystemBuilder::new("hit_point_regen")
        .read_resource::<FrameTime>()
        .with_query(<(::legion::Entity, ::legion::Write<HitPoints>)>::query())
        .build(move |cmd, world, resources, query| {
            let (mut for_query, mut world) = world.split_for_query(query);
            let for_query = &mut for_query;
            query.for_each_mut(for_query, |components| {
                hit_point_regen(&mut world, cmd, &*resources, components.0, components.1);
            });
        })
}
#[allow(dead_code)]
pub fn hit_point_regen(
    _world: &mut SubWorld,
    commands: &mut CommandBuffer,
    frame_time: &FrameTime,
    ent: &Entity,
    hp: &mut HitPoints,
) {
    if hp.health <= 0.0 {
        commands.remove_component::<AIFollow>(*ent);
        commands.remove_component::<Destination>(*ent);
    } else {
        hp.health += 0.7654321 * frame_time.0;
        hp.health = hp.max.min(hp.health);
    }
}

#[allow(unused)]
fn ai_follow_system() -> impl ParallelRunnable {
    SystemBuilder::new("ai_follow")
        .read_component::<AIFollow>()
        .read_component::<Position>()
        .write_component::<Destination>()
        .write_component::<Rotation>()
        .build(move |cmd, world, _resources, _query| {
            ai_follow(world, cmd);
        })
}

#[allow(dead_code)]
fn ai_follow(world: &mut SubWorld, command: &mut CommandBuffer) {
    let mut query = <(Entity, TryWrite<Rotation>, &AIFollow, &Position)>::query();
    let (mut hunter_world, hunted_world) = world.split_for_query(&query);
    for (ent, orient, follow, hunter) in query.iter_mut(&mut hunter_world) {
        if let Some(hunted) = hunted_world
            .entry_ref(follow.target)
            .ok()
            .map(|e| e.into_component::<Position>().ok())
            .flatten()
        {
            let difference: Vector3<f32> = hunted.0 - hunter.0;
            let distance = difference.magnitude();
            if distance > follow.minimum_distance {
                command.add_component(*ent, Destination::simple(hunted.0.truncate()));
                if let Some(orientation) = orient {
                    *orientation = Rotation::from(difference.angle(Vector3::unit_y()));
                }
            }
        }
    }
}

pub fn go_to_destination_system() -> impl ParallelRunnable {
    SystemBuilder::new("go_to_destination")
        .read_component::<Position>()
        .read_component::<Speed>()
        .read_component::<Acceleration>()
        .write_component::<Destination>()
        .write_component::<Velocity>()
        .read_resource::<FrameTime>()
        .build(move |cmd, world, resources, _query| {
            go_to_destination(world, cmd, &resources);
        })
}
#[allow(dead_code)]
pub fn go_to_destination(
    world: &mut SubWorld,
    commands: &mut legion::systems::CommandBuffer,
    frame_time: &FrameTime,
) {
    const EPSILON: f32 = 0.05;
    let mut query = <(
        Entity,
        &Destination,
        &Position,
        &mut Velocity,
        &Speed,
        &Acceleration,
    )>::query();
    for (ent, dest, hunter, vel, speed, accel) in query.iter_mut(world) {
        let to_dest: Vector2<f32> = dest.goal - hunter.0.truncate();
        if to_dest.magnitude() < EPSILON {
            commands.remove_component::<Destination>(*ent);
            vel.0 = Vector2::new(0.0, 0.0);
        } else {
            let direction = to_dest.normalize();
            let time_to_stop = speed.0 / accel.0;
            let slowdown = FRAC_PI_2
                .min(to_dest.magnitude() / time_to_stop * 0.5)
                .sin();
            let target_velocity = direction * speed.0 * slowdown;
            let delta: Vector2<f32> = target_velocity - vel.0;
            let velocity_change = (accel.0 * frame_time.0).min(delta.magnitude());
            if delta != Vector2::unit_x() * 0.0 {
                vel.0 += delta.normalize() * velocity_change;
            }
        }
    }
}
