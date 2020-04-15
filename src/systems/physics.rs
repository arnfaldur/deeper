use specs::prelude::*;

use cgmath::{prelude::*, Vector2};

use crate::components::*;

pub struct MovementSystem;

impl<'a> System<'a> for MovementSystem {
    type SystemData = (
        ReadExpect<'a, FrameTime>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, Velocity>,
    );

    fn run(&mut self, (frame_time, mut pos, mut vel): Self::SystemData) {
        for (pos, vel) in (&mut pos, &mut vel).join() {
            if vel.0.x.is_finite() && vel.0.y.is_finite() {
                let v = if (vel.0 * frame_time.0).magnitude() < 0.5 { vel.0 * frame_time.0 } else { (vel.0 * frame_time.0).normalize() * 0.5 };
                pos.0 += v;
            } else {
                // We need to deal with this somehow
                vel.0 = Vector2::new(0.0, 0.0);
                println!("Velocity Hickup");
            }
        }
    }
}

pub struct Physics2DSystem;

impl<'a> System<'a> for Physics2DSystem {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, FrameTime>,
        ReadStorage<'a, Position>,
        WriteStorage<'a, Velocity>,
        ReadStorage<'a, StaticBody>,
        ReadStorage<'a, DynamicBody>,
        ReadStorage<'a, CircleCollider>,
        ReadStorage<'a, SquareCollider>,
    );

    fn run(&mut self, (ents, frame_time, pos, mut vel, statics, dynamics, circles, squares): Self::SystemData) {
        // TODO: Move these to an EntityValidationSystem or something like that
        (&dynamics, &statics).par_join().for_each(|_| {
            panic!("There's a naughty static body that really feels dynamic inside.");
        });
        for _ in (&dynamics, !&vel).join() {
            panic!("A dynamic entity has no velocity!");
        }
        let damping = 4.0;
        for (_, vel) in (&dynamics, &mut vel).join() {
            //vel.0 *= 1. - (damping * frame_time.0).min(1.);
        }
        (&ents, &dynamics, &pos, &circles).join().for_each(|(ent_a, _, pos_a, circle_a)| {
            (&ents, &dynamics, &pos, &circles).join().for_each(|(ent_b, _, pos_b, circle_b)| {
                if ent_a != ent_b {
                    let collision_distance = circle_a.radius + circle_b.radius;

                    // get post move locations
                    let delta_a = pos_a.0 + frame_time.0 * vel.get(ent_a).unwrap().0;
                    let delta_b = pos_b.0 + frame_time.0 * vel.get(ent_b).unwrap().0;
                    // vector from ent_a to ent_b
                    let position_delta = delta_a - delta_b;
                    // how much are we colliding?
                    let collision_depth = collision_distance - position_delta.magnitude();
                    // same as position_delta but without velocity applied
                    let collision_direction = position_delta.normalize();
                    if collision_depth > 0.0 {
                        //vel.get_mut(ent_a).unwrap().0 += (position_delta.normalize_to(collision_depth));
                        //vel.get_mut(ent_b).unwrap().0 -= (position_delta.normalize_to(collision_depth));

                        // get_mut is necessary to appease the borrow checker
                        //vel.get_mut(ent_a).unwrap().0 += collision_direction * collision_depth / 2.0 / frame_time.0;
                        //vel.get_mut(ent_b).unwrap().0 -= collision_direction * collision_depth / 2.0 / frame_time.0;
                        let vela = vel.get(ent_a).unwrap().0;
                        let velb = vel.get(ent_b).unwrap().0;

                        let pa = vela.project_on(collision_direction);
                        let pb = velb.project_on(collision_direction);

                        let pia = pa.normalize() * pb.magnitude();
                        let pib = pb.normalize() * pa.magnitude();

                        if vela.dot(collision_direction) < 0.0 {
                            vel.get_mut(ent_a).unwrap().0 -= pa + 1.2 * pia;
                        }
                        if velb.dot(collision_direction) > 0.0 {
                            vel.get_mut(ent_b).unwrap().0 -= pb + 1.2 * pib;
                        }
                    }
                }
            });
        });
        (&ents, &dynamics, &pos, &circles).join().for_each(|(ent_a, _, pos_a, circle_a)| {
            (&ents, &dynamics, &pos, &circles).join().for_each(|(ent_b, _, pos_b, circle_b)| {
                if ent_a != ent_b {
                    let collision_distance = circle_a.radius + circle_b.radius;

                    // get post move locations
                    let delta_a = pos_a.0 + frame_time.0 * vel.get(ent_a).unwrap().0;
                    let delta_b = pos_b.0 + frame_time.0 * vel.get(ent_b).unwrap().0;
                    // vector from ent_a to ent_b
                    let position_delta = delta_a - delta_b;
                    // how much are we colliding?
                    let collision_depth = collision_distance - position_delta.magnitude();
                    // same as position_delta but without velocity applied
                    let collision_direction = position_delta.normalize();
                    if collision_depth > 0.0 {
                        //vel.get_mut(ent_a).unwrap().0 += (position_delta.normalize_to(collision_depth));
                        //vel.get_mut(ent_b).unwrap().0 -= (position_delta.normalize_to(collision_depth));

                        // get_mut is necessary to appease the borrow checker
                        vel.get_mut(ent_a).unwrap().0 += collision_direction * collision_depth / 1.0 / frame_time.0;
                        vel.get_mut(ent_b).unwrap().0 -= collision_direction * collision_depth / 1.0 / frame_time.0;
                    }
                }
            });
        });
        for (_, pos_a, vel_a, circle_a) in (&dynamics, &pos, &mut vel, &circles).join() {
            for (_, pos_b, circle_b) in (&statics, &pos, &circles).join() {
                let collision_distance = circle_a.radius + circle_b.radius;
                // get post move locations
                let delta_a = pos_a.0 + vel_a.0 * frame_time.0;
                let delta_b = pos_b.0;
                // vector from ent_a to ent_b
                let position_delta = delta_a - delta_b;
                // how much are we colliding?
                let collision_depth = collision_distance - position_delta.magnitude();
                if collision_depth > 0.0 {
                    vel_a.0 += position_delta.normalize_to(collision_depth);
                }
            }
        }
        (&dynamics, &pos, &mut vel, &circles).join().for_each(|(_, pos_a, vel_a, circle_a)| {
            for (_, pos_b, square_b) in (&statics, &pos, &squares).join() {
                let half_side = square_b.side_length / 2.0;
                let position_difference: Vector2<f32> = pos_a.0 + vel_a.0 * frame_time.0 - pos_b.0;
                let abs_pos_diff: Vector2<f32> = Vector2::new(position_difference.x.abs(), position_difference.y.abs());
                let difference_from_corner = abs_pos_diff - Vector2::new(half_side, half_side);
                if difference_from_corner.magnitude() < circle_a.radius {
                    let sigference: Vector2<f32> = Vector2::new(position_difference.x.signum(), position_difference.y.signum());
                    let vel_change = sigference.mul_element_wise(difference_from_corner.normalize()) * (difference_from_corner.magnitude() - circle_a.radius);
                    vel_a.0 -= vel_change / frame_time.0;
                }
                let diff: Vector2<f32> = pos_a.0 + vel_a.0 * frame_time.0 - pos_b.0;
                let abs_diff: Vector2<f32> = Vector2::new(diff.x.abs(), diff.y.abs());
                if abs_diff.x <= half_side {
                    if abs_diff.y < half_side + circle_a.radius {
                        vel_a.0.y -= (abs_diff.y - circle_a.radius - half_side) * diff.y.signum() / frame_time.0;
                    }
                }
                if abs_diff.y <= half_side {
                    if abs_diff.x < half_side + circle_a.radius {
                        vel_a.0.x -= (abs_diff.x - circle_a.radius - half_side) * diff.x.signum() / frame_time.0;
                    }
                }
            }
        });
    }
}
