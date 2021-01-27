use legion::*;

use cgmath::{prelude::*, Vector2};

use crate::components::*;
use legion::systems::Builder;
use legion::world::SubWorld;

pub(crate) trait PhysicsBuilderExtender {
    fn add_physics_systems(&mut self) -> &mut Self;
}

impl PhysicsBuilderExtender for Builder {
    fn add_physics_systems(&mut self) -> &mut Self {
        self.add_system(pre_physics_2d_stat_dyna_check_system())
            .add_system(pre_physics_2d_dyna_vel_check_system())
            .add_system(init_velocity_accumulator_system())
            .add_system(physics_2d_1_system())
            .add_system(velocity_update_system())
            .add_system(init_velocity_accumulator_system())
            .add_system(physics_2d_2_system())
            .add_system(velocity_update_system())
            .add_system(init_velocity_accumulator_system())
            .add_system(physics_2d_3_system())
            .add_system(velocity_update_system())
            .add_system(init_velocity_accumulator_system())
            .add_system(physics_2d_4_system())
            .add_system(velocity_update_system())
            .add_system(movement_system())
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

// TODO: replace this whole mess with a real physics engine
fn accumulate_velocity(world: &mut SubWorld, entity: &Entity, vector: Vector2<f32>) {
    if let Result::Ok(mut entry) = world.entry_mut(*entity) {
        if let Result::Ok(mut vel_acc) = entry.get_component_mut::<VelocityAccumulator>() {
            vel_acc.accumulate(vector);
        }
    }
}

#[system]
#[read_component(Position)]
#[read_component(Velocity)]
#[write_component(VelocityAccumulator)]
#[read_component(DynamicBody)]
#[read_component(CircleCollider)]
pub fn physics_2d_1(world: &mut SubWorld, #[resource] frame_time: &FrameTime) {
    let mut query_a = <(Entity, &Position, &Velocity, &DynamicBody, &CircleCollider)>::query()
        .filter(component::<VelocityAccumulator>());
    let mut query_b = <(Entity, &Position, &Velocity, &DynamicBody, &CircleCollider)>::query()
        .filter(component::<VelocityAccumulator>());
    let (mut changing, comparing) = world.split::<Write<VelocityAccumulator>>();
    for (ent_a, pos_a, vel_a, dyn_a, circle_a) in query_a.iter(&comparing) {
        for (ent_b, pos_b, vel_b, dyn_b, circle_b) in query_b.iter(&comparing) {
            if ent_a != ent_b {
                let collision_distance = circle_a.radius + circle_b.radius;

                // get post move locations
                let delta_a = pos_a.0 + frame_time.0 * vel_a.0;
                let delta_b = pos_b.0 + frame_time.0 * vel_b.0;
                // vector from ent_a to ent_b
                let position_delta = delta_a - delta_b;
                // how much are we colliding?
                let collision_depth = collision_distance - position_delta.magnitude();
                if collision_depth > 0.0 {
                    // are they colliding?
                    // same as position_delta but without velocity applied
                    let collision_direction = position_delta.normalize();
                    // how much are they moving into each other?
                    let pa = vel_a.0.project_on(collision_direction);
                    let pb = vel_b.0.project_on(collision_direction);
                    // flipping sides or something?
                    let pia = pa.normalize() * pb.magnitude();
                    let pib = pb.normalize() * pa.magnitude();

                    if vel_a.0.dot(collision_direction) < 0.0 {
                        accumulate_velocity(&mut changing, ent_a, vel_a.0 - (pa + 1.2 * pia));
                    }
                    if vel_b.0.dot(collision_direction) > 0.0 {
                        accumulate_velocity(&mut changing, ent_b, vel_b.0 - (pb + 1.2 * pib));
                    }
                }
            }
        }
    }
}

#[system]
#[read_component(Position)]
#[read_component(Velocity)]
#[write_component(VelocityAccumulator)]
#[read_component(DynamicBody)]
#[read_component(CircleCollider)]
pub fn physics_2d_2(world: &mut SubWorld, #[resource] frame_time: &FrameTime) {
    let mut query_a = <(Entity, &Position, &Velocity, &DynamicBody, &CircleCollider)>::query()
        .filter(component::<VelocityAccumulator>());
    let mut query_b = <(Entity, &Position, &Velocity, &DynamicBody, &CircleCollider)>::query()
        .filter(component::<VelocityAccumulator>());
    let (mut changing, comparing) = world.split::<Write<VelocityAccumulator>>();
    for (ent_a, pos_a, vel_a, dyn_a, circle_a) in query_a.iter(&comparing) {
        for (ent_b, pos_b, vel_b, dyn_b, circle_b) in query_b.iter(&comparing) {
            if ent_a != ent_b {
                let collision_distance = circle_a.radius + circle_b.radius;

                // get post move locations
                let delta_a = pos_a.0 + frame_time.0 * vel_a.0;
                let delta_b = pos_b.0 + frame_time.0 * vel_b.0;
                // vector from ent_a to ent_b
                let position_delta = delta_a - delta_b;
                // how much are we colliding?
                let collision_depth = collision_distance - position_delta.magnitude();
                if collision_depth > 0.0 {
                    // same as position_delta but without velocity applied
                    let collision_direction = position_delta.normalize();

                    accumulate_velocity(
                        &mut changing,
                        ent_a,
                        vel_a.0 + (collision_direction * collision_depth / 1.0 / frame_time.0),
                    );
                    accumulate_velocity(
                        &mut changing,
                        ent_b,
                        vel_b.0 - (collision_direction * collision_depth / 1.0 / frame_time.0),
                    );
                }
            }
        }
    }
}

#[system]
#[read_component(Position)]
#[read_component(Velocity)]
#[write_component(VelocityAccumulator)]
#[read_component(DynamicBody)]
#[read_component(CircleCollider)]
pub fn physics_2d_3(world: &mut SubWorld, #[resource] frame_time: &FrameTime) {
    let mut query_a = <(Entity, &Position, &Velocity, &DynamicBody, &CircleCollider)>::query()
        .filter(component::<VelocityAccumulator>());
    let mut query_b = <(&Position, &DynamicBody, &CircleCollider)>::query()
        .filter(component::<VelocityAccumulator>());
    let (mut changing, comparing) = world.split::<Write<VelocityAccumulator>>();
    for (ent_a, pos_a, vel_a, _, circle_a) in query_a.iter(&comparing) {
        for (pos_b, _, circle_b) in query_b.iter(&comparing) {
            let collision_distance = circle_a.radius + circle_b.radius;
            // get post move locations
            let delta_a = pos_a.0 + vel_a.0 * frame_time.0;
            let delta_b = pos_b.0;
            // vector from ent_a to ent_b
            let position_delta = delta_a - delta_b;
            // how much are we colliding?
            let collision_depth = collision_distance - position_delta.magnitude();
            if collision_depth > 0.0 {
                accumulate_velocity(
                    &mut changing,
                    ent_a,
                    vel_a.0 + position_delta.normalize_to(collision_depth),
                );
            }
        }
    }
}

#[system]
#[read_component(Position)]
#[read_component(Velocity)]
#[write_component(VelocityAccumulator)]
#[read_component(StaticBody)]
#[read_component(DynamicBody)]
#[read_component(CircleCollider)]
#[read_component(SquareCollider)]
pub fn physics_2d_4(world: &mut SubWorld, #[resource] frame_time: &FrameTime) {
    let mut query_a = <(Entity, &Position, &Velocity, &DynamicBody, &CircleCollider)>::query()
        .filter(component::<VelocityAccumulator>());
    let mut query_b = <(&Position, &SquareCollider)>::query().filter(component::<StaticBody>());
    let (mut changing, comparing) = world.split::<Write<VelocityAccumulator>>();

    for (ent_a, pos_a, vel_a, dyn_a, circle_a) in query_a.iter(&comparing) {
        for (pos_b, square_b) in query_b.iter(&comparing) {
            let half_side = square_b.side_length / 2.0;
            let position_difference: Vector2<f32> = pos_a.0 + vel_a.0 * frame_time.0 - pos_b.0;
            let abs_pos_diff: Vector2<f32> =
                Vector2::new(position_difference.x.abs(), position_difference.y.abs());
            let difference_from_corner = abs_pos_diff - Vector2::new(half_side, half_side);
            if difference_from_corner.magnitude() < circle_a.radius {
                let sigference: Vector2<f32> = Vector2::new(
                    position_difference.x.signum(),
                    position_difference.y.signum(),
                );
                let vel_change = sigference.mul_element_wise(difference_from_corner.normalize())
                    * (difference_from_corner.magnitude() - circle_a.radius);
                accumulate_velocity(&mut changing, ent_a, vel_a.0 - (vel_change / frame_time.0));
            }
            let diff: Vector2<f32> = pos_a.0 + vel_a.0 * frame_time.0 - pos_b.0;
            let abs_diff: Vector2<f32> = Vector2::new(diff.x.abs(), diff.y.abs());
            let mut target = vel_a.0;
            if abs_diff.x <= half_side {
                if abs_diff.y < half_side + circle_a.radius {
                    target.y -=
                        (abs_diff.y - circle_a.radius - half_side) * diff.y.signum() / frame_time.0;
                }
            }
            if abs_diff.y <= half_side {
                if abs_diff.x < half_side + circle_a.radius {
                    target.x -=
                        (abs_diff.x - circle_a.radius - half_side) * diff.x.signum() / frame_time.0;
                }
            }
            accumulate_velocity(&mut changing, ent_a, target);
        }
    }
    // (&dynamics, &pos, &mut vel, &circles) .join()
    //     .for_each(|(_, pos_a, vel_a, circle_a)| {
    //         for (_, pos_b, square_b) in (&statics, &pos, &squares).join() {
    //             let half_side = square_b.side_length / 2.0;
    //             let position_difference: Vector2<f32> = pos_a.0 + vel_a.0 * frame_time.0 - pos_b.0;
    //             let abs_pos_diff: Vector2<f32> =
    //                 Vector2::new(position_difference.x.abs(), position_difference.y.abs());
    //             let difference_from_corner = abs_pos_diff - Vector2::new(half_side, half_side);
    //             if difference_from_corner.magnitude() < circle_a.radius {
    //                 let sigference: Vector2<f32> = Vector2::new(
    //                     position_difference.x.signum(),
    //                     position_difference.y.signum(),
    //                 );
    //                 let vel_change = sigference
    //                     .mul_element_wise(difference_from_corner.normalize())
    //                     * (difference_from_corner.magnitude() - circle_a.radius);
    //                 vel_a.0 -= vel_change / frame_time.0;
    //             }
    //             let diff: Vector2<f32> = pos_a.0 + vel_a.0 * frame_time.0 - pos_b.0;
    //             let abs_diff: Vector2<f32> = Vector2::new(diff.x.abs(), diff.y.abs());
    //             if abs_diff.x <= half_side {
    //                 if abs_diff.y < half_side + circle_a.radius {
    //                     vel_a.0.y -= (abs_diff.y - circle_a.radius - half_side) * diff.y.signum()
    //                         / frame_time.0;
    //                 }
    //             }
    //             if abs_diff.y <= half_side {
    //                 if abs_diff.x < half_side + circle_a.radius {
    //                     vel_a.0.x -= (abs_diff.x - circle_a.radius - half_side) * diff.x.signum()
    //                         / frame_time.0;
    //                 }
    //             }
    //         }
    //     });
}
// pub struct Physics2DSystem;
//
// impl<'a> System<'a> for Physics2DSystem {
//     type SystemData = (
//         Entities<'a>,
//         ReadExpect<'a, FrameTime>,
//         ReadStorage<'a, Position>,
//         WriteStorage<'a, Velocity>,
//         ReadStorage<'a, StaticBody>,
//         ReadStorage<'a, DynamicBody>,
//         ReadStorage<'a, CircleCollider>,
//         ReadStorage<'a, SquareCollider>,
//     );
//
//     fn run(&mut self, (ents, frame_time, pos, mut vel, statics, dynamics, circles, squares): Self::SystemData) {
//         // TODO: Move these to an EntityValidationSystem or something like that
//         (&dynamics, &statics).par_join().for_each(|_| {
//             panic!("There's a naughty static body that really feels dynamic inside.");
//         });
//         for _ in (&dynamics, !&vel).join() {
//             panic!("A dynamic entity has no velocity!");
//         }
//         let damping = 4.0;
//         for (_, vel) in (&dynamics, &mut vel).join() {
//             //vel.0 *= 1. - (damping * frame_time.0).min(1.);
//         }
//         (&ents, &dynamics, &pos, &circles).join().for_each(|(ent_a, _, pos_a, circle_a)| {
//             (&ents, &dynamics, &pos, &circles).join().for_each(|(ent_b, _, pos_b, circle_b)| {
//                 if ent_a != ent_b {
//                     let collision_distance = circle_a.radius + circle_b.radius;
//
//                     // get post move locations
//                     let delta_a = pos_a.0 + frame_time.0 * vel.get(ent_a).unwrap().0;
//                     let delta_b = pos_b.0 + frame_time.0 * vel.get(ent_b).unwrap().0;
//                     // vector from ent_a to ent_b
//                     let position_delta = delta_a - delta_b;
//                     // how much are we colliding?
//                     let collision_depth = collision_distance - position_delta.magnitude();
//                     // same as position_delta but without velocity applied
//                     let collision_direction = position_delta.normalize();
//                     if collision_depth > 0.0 {
//                         //vel.get_mut(ent_a).unwrap().0 += (position_delta.normalize_to(collision_depth));
//                         //vel.get_mut(ent_b).unwrap().0 -= (position_delta.normalize_to(collision_depth));
//
//                         // get_mut is necessary to appease the borrow checker
//                         //vel.get_mut(ent_a).unwrap().0 += collision_direction * collision_depth / 2.0 / frame_time.0;
//                         //vel.get_mut(ent_b).unwrap().0 -= collision_direction * collision_depth / 2.0 / frame_time.0;
//                         let vela = vel.get(ent_a).unwrap().0;
//                         let velb = vel.get(ent_b).unwrap().0;
//
//                         let pa = vela.project_on(collision_direction);
//                         let pb = velb.project_on(collision_direction);
//
//                         let pia = pa.normalize() * pb.magnitude();
//                         let pib = pb.normalize() * pa.magnitude();
//
//                         if vela.dot(collision_direction) < 0.0 {
//                             vel.get_mut(ent_a).unwrap().0 -= pa + 1.2 * pia;
//                         }
//                         if velb.dot(collision_direction) > 0.0 {
//                             vel.get_mut(ent_b).unwrap().0 -= pb + 1.2 * pib;
//                         }
//                     }
//                 }
//             });
//         });
//         (&ents, &dynamics, &pos, &circles).join().for_each(|(ent_a, _, pos_a, circle_a)| {
//             (&ents, &dynamics, &pos, &circles).join().for_each(|(ent_b, _, pos_b, circle_b)| {
//                 if ent_a != ent_b {
//                     let collision_distance = circle_a.radius + circle_b.radius;
//
//                     // get post move locations
//                     let delta_a = pos_a.0 + frame_time.0 * vel.get(ent_a).unwrap().0;
//                     let delta_b = pos_b.0 + frame_time.0 * vel.get(ent_b).unwrap().0;
//                     // vector from ent_a to ent_b
//                     let position_delta = delta_a - delta_b;
//                     // how much are we colliding?
//                     let collision_depth = collision_distance - position_delta.magnitude();
//                     // same as position_delta but without velocity applied
//                     let collision_direction = position_delta.normalize();
//                     if collision_depth > 0.0 {
//                         //vel.get_mut(ent_a).unwrap().0 += (position_delta.normalize_to(collision_depth));
//                         //vel.get_mut(ent_b).unwrap().0 -= (position_delta.normalize_to(collision_depth));
//
//                         // get_mut is necessary to appease the borrow checker
//                         vel.get_mut(ent_a).unwrap().0 += collision_direction * collision_depth / 1.0 / frame_time.0;
//                         vel.get_mut(ent_b).unwrap().0 -= collision_direction * collision_depth / 1.0 / frame_time.0;
//                     }
//                 }
//             });
//         });
//         for (_, pos_a, vel_a, circle_a) in (&dynamics, &pos, &mut vel, &circles).join() {
//             for (_, pos_b, circle_b) in (&statics, &pos, &circles).join() {
//                 let collision_distance = circle_a.radius + circle_b.radius;
//                 // get post move locations
//                 let delta_a = pos_a.0 + vel_a.0 * frame_time.0;
//                 let delta_b = pos_b.0;
//                 // vector from ent_a to ent_b
//                 let position_delta = delta_a - delta_b;
//                 // how much are we colliding?
//                 let collision_depth = collision_distance - position_delta.magnitude();
//                 if collision_depth > 0.0 {
//                     vel_a.0 += position_delta.normalize_to(collision_depth);
//                 }
//             }
//         }
//         (&dynamics, &pos, &mut vel, &circles).join().for_each(|(_, pos_a, vel_a, circle_a)| {
//             for (_, pos_b, square_b) in (&statics, &pos, &squares).join() {
//                 let half_side = square_b.side_length / 2.0;
//                 let position_difference: Vector2<f32> = pos_a.0 + vel_a.0 * frame_time.0 - pos_b.0;
//                 let abs_pos_diff: Vector2<f32> = Vector2::new(position_difference.x.abs(), position_difference.y.abs());
//                 let difference_from_corner = abs_pos_diff - Vector2::new(half_side, half_side);
//                 if difference_from_corner.magnitude() < circle_a.radius {
//                     let sigference: Vector2<f32> = Vector2::new(position_difference.x.signum(), position_difference.y.signum());
//                     let vel_change = sigference.mul_element_wise(difference_from_corner.normalize()) * (difference_from_corner.magnitude() - circle_a.radius);
//                     vel_a.0 -= vel_change / frame_time.0;
//                 }
//                 let diff: Vector2<f32> = pos_a.0 + vel_a.0 * frame_time.0 - pos_b.0;
//                 let abs_diff: Vector2<f32> = Vector2::new(diff.x.abs(), diff.y.abs());
//                 if abs_diff.x <= half_side {
//                     if abs_diff.y < half_side + circle_a.radius {
//                         vel_a.0.y -= (abs_diff.y - circle_a.radius - half_side) * diff.y.signum() / frame_time.0;
//                     }
//                 }
//                 if abs_diff.y <= half_side {
//                     if abs_diff.x < half_side + circle_a.radius {
//                         vel_a.0.x -= (abs_diff.x - circle_a.radius - half_side) * diff.x.signum() / frame_time.0;
//                     }
//                 }
//             }
//         });
//     }
// }

#[system(for_each)]
pub fn init_velocity_accumulator(vel: &Velocity, velacc: &mut VelocityAccumulator) {
    *velacc = VelocityAccumulator::zero();
}

#[system(for_each)]
pub fn velocity_update(vel: &mut Velocity, velacc: &VelocityAccumulator) {
    vel.0 = velacc.get();
}
