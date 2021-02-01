use std::f32::consts::FRAC_PI_2;

use cgmath::{prelude::*, Vector2};

use crate::components::*;

pub mod assets;
pub mod physics;
pub mod player;
pub mod rendering;
pub mod world_gen;

use legion::systems::CommandBuffer;
use legion::world::SubWorld;
use legion::*;

#[system(for_each)]
pub fn spherical_offset(pos2d: &Position, follow: &SphericalOffset, pos3d: &mut Position3D) {
    pos3d.0 = pos2d.into();

    pos3d.0.x += follow.radius * follow.theta.cos() * follow.phi.cos();
    pos3d.0.y += follow.radius * follow.theta.sin() * follow.phi.cos();
    pos3d.0.z += follow.radius * follow.phi.sin();
}

#[system(for_each)]
pub fn hit_point_regen(
    world: &mut SubWorld,
    commands: &mut CommandBuffer,
    #[resource] frame_time: &FrameTime,
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

#[system]
#[write_component(Destination)]
#[write_component(Orientation)]
#[read_component(AIFollow)]
#[read_component(Position)]
fn ai_follow(world: &mut SubWorld, command: &mut CommandBuffer) {
    let mut query = <(Entity, TryWrite<Orientation>, &AIFollow, &Position)>::query();
    let (mut hunter_world, mut hunted_world) = world.split_for_query(&query);
    for (ent, orient, follow, hunter) in query.iter_mut(&mut hunter_world) {
        if let Some(hunted) = hunted_world
            .entry_ref(follow.target)
            .ok()
            .map(|e| e.into_component::<Position>().ok())
            .flatten()
        {
            let difference: Vector2<f32> = hunted.0 - hunter.0;
            let distance = difference.magnitude();
            if distance > follow.minimum_distance {
                command.add_component(*ent, Destination::simple(hunted.0));
                if let Some(orientation) = orient {
                    orientation.0 = cgmath::Deg::from(difference.angle(Vector2::unit_y()));
                }
            }
        }
    }
}

#[system]
#[write_component(Destination)]
#[read_component(Position)]
#[write_component(Velocity)]
#[read_component(Speed)]
#[read_component(Acceleration)]
pub fn go_to_destination(
    world: &mut SubWorld,
    commands: &mut legion::systems::CommandBuffer,
    #[resource] frame_time: &FrameTime,
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
        // check if straight path is available, line drawing? or just navmesh
        // if not do A* and add intermediate destination component for next node in path
        // or just make Destination an object inheriting from the abstract destinations
        // class.
        let to_dest: Vector2<f32> = dest.goal - hunter.0;

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

//pub struct IntermediateDestinationSystem;
//
//impl<'a> System<'a> for IntermediateDestinationSystem {
//    type SystemData = (
//
//    );
//
//    fn run(&mut self, (): Self::SystemData) {
//
//    }
//}
