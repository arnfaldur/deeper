use specs::prelude::*;
use std::f32::consts::PI;
use std::ops::Mul;
use zerocopy::AsBytes;

extern crate cgmath;

use cgmath::{prelude::*, Vector2, Vector3};

use crate::graphics;
use crate::components::components::*;

pub struct MovementSystem;

impl<'a> System<'a> for MovementSystem {
    type SystemData = (WriteStorage<'a, Position>, ReadStorage<'a, Velocity>);

    fn run(&mut self, (mut pos, vel): Self::SystemData) {
        let mut rng = thread_rng();
        for (pos, vel) in (&mut pos, &vel).join() {
            let velo: Vector2<f32> = vel.0;
            let (randx, randy): (f32, f32) = (
                rng.gen_range(-std::f32::EPSILON * 10.0, std::f32::EPSILON * 10.0),
                rng.gen_range(-std::f32::EPSILON * 10.0, std::f32::EPSILON * 10.0)
            );
            pos.0 += Vector2::new(velo.x + randx, velo.y + randy);
        }
    }
}

pub struct SphericalFollowSystem;

impl<'a> System<'a> for SphericalFollowSystem {
    type SystemData = (
        ReadStorage<'a, Position>,
        ReadStorage<'a, Target>,
        ReadStorage<'a, SphericalOffset>,
        WriteStorage<'a, Position3D>,
    );

    fn run(&mut self, (pos2d, target, follow, mut pos3d): Self::SystemData) {
        for (target, follow, pos3d) in (&target, &follow, &mut pos3d).join() {
            pos3d.0 = pos2d.get(target.0).cloned().unwrap().to_vec3();
            pos3d.0.x += follow.radius * follow.theta.cos() * follow.phi.cos();
            pos3d.0.y += follow.radius * follow.theta.sin() * follow.phi.cos();
            pos3d.0.z += follow.radius * follow.phi.sin();
        }
    }
}

pub struct GraphicsSystem {
    pub model_array: Vec<graphics::Model>,
}

impl GraphicsSystem {
    pub fn new(
        model_array: Vec<graphics::Model>,
    ) -> Self {
        Self { model_array }
    }
}

impl<'a> System<'a> for GraphicsSystem {
    type SystemData = (
        WriteExpect<'a, graphics::Context>,
        ReadExpect<'a, ActiveCamera>,

        ReadStorage<'a, Camera>,
        ReadStorage<'a, Target>,
        ReadStorage<'a, Position3D>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Orientation>,
        ReadStorage<'a, Model3D>,
        ReadStorage<'a, StaticModel>,
    );

    fn run(&mut self, (mut context, active_cam, camera, target, pos3d, pos, orient, models, static_model): Self::SystemData) {
        let frame = context.swap_chain.get_next_texture().unwrap();

        let cam = camera.get(active_cam.0)
            .expect("No valid active camera entity");

        let cam_pos = pos3d.get(active_cam.0)
            .expect("Camera entity has no 3D position");

        let cam_target =
            pos.get(target.get(active_cam.0).unwrap().0).unwrap().to_vec3();

        let mx_correction = graphics::correction_matrix();

        let mx_view = cgmath::Matrix4::look_at(
            graphics::to_pos3(cam_pos.0),
            graphics::to_pos3(cam_target),
            cgmath::Vector3::unit_z(),
        );

        let mx_projection = cgmath::perspective(
            cgmath::Deg(cam.fov),
            context.sc_desc.width as f32 / context.sc_desc.height as f32,
            1.0,
            1000.0,
        );

        let mx = mx_correction * mx_projection * mx_view;

        let global_uniforms = graphics::GlobalUniforms {
            projection_view_matrix: mx.into(),
            eye_position: [cam_pos.0.x, cam_pos.0.y, cam_pos.0.z, 1.0],
        };

        let new_uniform_buf = context.device.create_buffer_with_data(
            global_uniforms.as_bytes(),
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_SRC,
        );

        let mut encoder = context.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor { todo: 0 }
        );

        encoder.copy_buffer_to_buffer(
            &new_uniform_buf,
            0,
            &context.uniform_buf,
            0,
            std::mem::size_of::<graphics::GlobalUniforms>() as u64,
        );

        let mut uniforms = vec!();

        for (pos, model, rotation) in (&pos, &models, (&orient).maybe()).join() {
            let mut matrix = Matrix4::from_scale(model.scale);
            if let Some(rot) = rotation {
                matrix = Matrix4::from_angle_z(cgmath::Deg(rot.0)) * matrix;
            }
            matrix = Matrix4::from_translation(pos.to_vec3()) * matrix;
            let local_uniforms = graphics::LocalUniforms {
                model_matrix: matrix.into(),
                color: model.tint.into(),
            };
            uniforms.push(local_uniforms);
        }

        let temp_buf = context.device.create_buffer_with_data(
            uniforms.as_bytes(),
            wgpu::BufferUsage::COPY_SRC,
        );

        for (i, (pos, model)) in (&pos, &models).join().enumerate() {
            encoder.copy_buffer_to_buffer(
                &temp_buf,
                (i * std::mem::size_of::<graphics::LocalUniforms>()) as u64,
                &model.uniform_buffer,
                0,
                std::mem::size_of::<graphics::LocalUniforms>() as u64,
            );
        }

        {
            let tic = SystemTime::now();
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    load_op: wgpu::LoadOp::Clear,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 },
                }],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &context.depth_view,
                    depth_load_op: wgpu::LoadOp::Clear,
                    depth_store_op: wgpu::StoreOp::Store,
                    clear_depth: 1.0,
                    stencil_load_op: wgpu::LoadOp::Clear,
                    stencil_store_op: wgpu::StoreOp::Store,
                    clear_stencil: 0,
                }),
            });

            rpass.set_pipeline(&context.pipeline);
            rpass.set_bind_group(0, &context.bind_group, &[]);

            for (model) in (&static_model).join() {
                rpass.set_bind_group(1, &model.bind_group, &[]);
                for mesh in &self.model_array[model.idx].meshes {
                    rpass.set_vertex_buffer(0, &mesh.vertex_buffer, 0, 0);
                    rpass.draw(0..mesh.num_vertices as u32, 0..1)
                }
            }

            for (_, model) in (&pos, &models).join() {
                rpass.set_bind_group(1, &model.bind_group, &[]);
                for mesh in &self.model_array[model.idx].meshes {
                    rpass.set_vertex_buffer(0, &mesh.vertex_buffer, 0, 0);
                    rpass.draw(0..mesh.num_vertices as u32, 0..1)
                }
            }
        }

        let command_buf = encoder.finish();

        context.queue.submit(&[command_buf]);
    }

    fn setup(&mut self, world: &mut World) {}
}

pub struct PlayerSystem {
    // Note(Jökull): Yeah, I know. This is just while we're feeling out what is the
    //               responsibility of the input handling system exactly
    last_mouse_pos: Vector2<f32>,
}

impl PlayerSystem {
    pub fn new() -> Self { Self { last_mouse_pos: Vector2::new(0.0, 0.0) } }
}

// Note(Jökull): Is this really just the input handler?
impl<'a> System<'a> for PlayerSystem {
    type SystemData = (
        ReadExpect<'a, InputState>,
        ReadExpect<'a, Player>,
        ReadExpect<'a, PlayerCamera>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Position3D>,
        ReadStorage<'a, Camera>,
        WriteStorage<'a, Orientation>,
        WriteStorage<'a, Model3D>,
        WriteStorage<'a, Destination>,
        WriteStorage<'a, SphericalOffset>,
    );

    fn run(&mut self, (input, player, player_cam, pos, pos3d, cam, mut orient, mut model, mut dest, mut offset): Self::SystemData) {
        let camera = cam.get(player_cam.0).unwrap();
        let camera_pos = pos3d.get(player_cam.0).unwrap();
        let mut camera_offset = offset.get_mut(player_cam.0).unwrap();

        //let mouse_delta = rl.get_mouse_position() - self.last_mouse_pos;
        let mouse_pos = input.mouse.pos;
        let mouse_delta = input.mouse.pos - self.last_mouse_pos;
        self.last_mouse_pos = input.mouse.pos;

        if input.mouse.middle.down {
            camera_offset.theta += camera_offset.theta_delta * mouse_delta.x;
            camera_offset.phi += camera_offset.phi_delta * mouse_delta.y;
            camera_offset.phi = camera_offset.phi.max(0.1 * PI).min(0.25 * PI);
        }

        let player_pos = pos.get(player.entity)
            .expect("I have no place in this world.");
        let mut player_orient = orient.get_mut(player.entity)
            .expect("We have no direction in life.");

        if input.mouse.left.down {
            // Note(Jökull): We need a better solution for this

            let mx_view = cgmath::Matrix4::look_at(
                graphics::to_pos3(camera_pos.0),
                graphics::to_pos3(player_pos.to_vec3()),
                cgmath::Vector3::unit_z(),
            );
            let mx_projection = cgmath::perspective(
                cgmath::Deg(camera.fov),
                1920f32 / 1080f32,
                1.0,
                1000.0,
            );

            if let Some(mouse_world_pos) = project_screen_to_world(
                Vector3::new(mouse_pos.x, 1080.0 - mouse_pos.y, 1.0),
                graphics::correction_matrix() * mx_projection * mx_view,
                Vector4::new(0, 0, 1920, 1080),
            ) {
                let ray_delta: Vector3<f32> = mouse_world_pos - camera_pos.0;
                let t: f32 = mouse_world_pos.z / ray_delta.z;
                let ray_hit = to_vec2(mouse_world_pos - ray_delta * t);

                dest.insert(player.entity, Destination(ray_hit));

                let difference: Vector2<f32> = (ray_hit - player_pos.0).normalize();

                let model = model.get_mut(player.entity).unwrap();
                let mut new_rotation = (difference.y / difference.x).atan() / PI * 180.0;
                if difference.x > 0.0 {
                    new_rotation += 180.0;
                }
                player_orient.0 = new_rotation;
                model.z_rotation = new_rotation;
            }
        }
    }

    fn setup(&mut self, world: &mut World) {
        println!("PlayerSystem setup!");
    }
}

pub(crate) struct AIFollowSystem;

impl<'a> System<'a> for AIFollowSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, AIFollow>,
        ReadStorage<'a, Position>,
        WriteStorage<'a, Destination>,
    );

    fn run(&mut self, (ents, follow, pos, mut dest): Self::SystemData) {
        for (ent, follow, hunter) in (&ents, &follow, &pos).join() {
            if let Some(hunted) = pos.get(follow.target) {
                let difference = hunted.0 - hunter.0;
                let distance = difference.magnitude();
                if distance > follow.minimum_distance {
                    dest.insert(ent, Destination(hunted.0));
                } else {
                    //dest.remove(ent);
                }
            }
        }
    }
}

pub(crate) struct GoToDestinationSystem;

impl<'a> System<'a> for GoToDestinationSystem {
    type SystemData = (
        ReadStorage<'a, Destination>,
        ReadStorage<'a, Position>,
        WriteStorage<'a, Velocity>,
        ReadStorage<'a, Speed>,
        ReadStorage<'a, Acceleration>,
    );

    fn run(&mut self, (dest, pos, mut vel, speed, acc): Self::SystemData) {
        for (dest, hunter, vel, speed, accel) in (&dest, &pos, &mut vel, &speed, &acc).join() {
            let to_dest: Vector2<f32> = dest.0 - hunter.0;
            let direction = to_dest.normalize();
            let accel_ratio = speed.0 / accel.0;
            let target_velocity = direction * if to_dest.magnitude() > accel.0 * (accel_ratio * (accel_ratio + 1.0) / 2.0) {
                speed.0
            } else {
                0.0
            };
            let delta = target_velocity - vel.0;
            vel.0 += delta.normalize() * accel.0;
            if delta.magnitude() < accel.0 {
                vel.0 = target_velocity;
            }
        }
    }
}

pub struct Physics2DSystem;

impl<'a> System<'a> for Physics2DSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Position>,
        WriteStorage<'a, Velocity>,
        ReadStorage<'a, StaticBody>,
        ReadStorage<'a, DynamicBody>,
        ReadStorage<'a, CircleCollider>,
        ReadStorage<'a, SquareCollider>,
    );

    fn run(&mut self, (ents, pos, mut vel, statics, dynamics, circles, squares): Self::SystemData) {
        // TODO: Move these to a EntityValidationSystem or something like that
        for _ in (&dynamics, &statics).join() {
            panic!("There's a naughty static body that really feels dynamic inside.");
        }
        for _ in (&dynamics, !&vel).join() {
            panic!("A dynamic entity has no velocity!");
        }
        for (ent_a, _, pos_a, circle_a) in (&ents, &dynamics, &pos, &circles).join() {
            for (ent_b, _, pos_b, circle_b) in (&ents, &dynamics, &pos, &circles).join() {
                if ent_a != ent_b {
                    let collision_distance = circle_a.radius + circle_b.radius;
                    // get post move locations
                    let delta_a = pos_a.0 + vel.get(ent_a).unwrap().0;
                    let delta_b = pos_b.0 + vel.get(ent_b).unwrap().0;
                    // vector from ent_a to ent_b
                    let position_delta = delta_a - delta_b;
                    // how much are we colliding?
                    let collision_depth = collision_distance - position_delta.magnitude();
                    if 0.0 < collision_depth {
                        // normalize the vector to scale and reflect
                        let collision_direction = position_delta.normalize();
                        // get_mut is necessary to appease the borrow checker
                        vel.get_mut(ent_a).unwrap().0 += collision_direction * collision_depth / 2.0;
                        vel.get_mut(ent_b).unwrap().0 += collision_direction * -collision_depth / 2.0;
                    }
                }
            }
        }
        for (_, pos_a, vel_a, circle_a) in (&dynamics, &pos, &mut vel, &circles).join() {
            for (_, pos_b, circle_b) in (&statics, &pos, &circles).join() {
                if (pos_a.0 + vel_a.0 - pos_b.0).magnitude() < (circle_a.radius + circle_b.radius) {
                    let diff = pos_b.0 - pos_a.0;
                    let collinear_part = diff * (vel_a.0.dot(diff));
                    vel_a.0 -= collinear_part;
                }
            }
        }
        for (_, pos_a, vel_a, circle_a) in (&dynamics, &pos, &mut vel, &circles).join() {
            for (_, pos_b, square_b) in (&statics, &pos, &squares).join() {
                let half_side = square_b.side_length / 2.0;
                let diff: Vector2<f32> = pos_a.0 + vel_a.0 - pos_b.0;
                let abs_diff: Vector2<f32> = Vector2::new(diff.x.abs(), diff.y.abs());
                let corner_dist = abs_diff - Vector2::new(half_side, half_side);
                if corner_dist.magnitude() < circle_a.radius {
                    let sigference: Vector2<f32> = Vector2::new(diff.x.signum(), diff.y.signum());
                    let vel_change = sigference.mul_element_wise(corner_dist.normalize()) * (corner_dist.magnitude() - circle_a.radius);
                    vel_a.0 -= vel_change;
                }
                let diff: Vector2<f32> = pos_a.0 + vel_a.0 - pos_b.0;
                let abs_diff: Vector2<f32> = Vector2::new(diff.x.abs(), diff.y.abs());
                if abs_diff.x <= half_side {
                    if abs_diff.y < half_side + circle_a.radius {
                        vel_a.0.y -= (abs_diff.y - circle_a.radius - half_side) * diff.y.signum();
                    }
                }
                if abs_diff.y <= half_side {
                    if abs_diff.x < half_side + circle_a.radius {
                        vel_a.0.x -= (abs_diff.x - circle_a.radius - half_side) * diff.x.signum();
                    }
                }
            }
        }
    }
}

use self::cgmath::{Matrix4, Vector4};
use crate::input::InputState;
use crate::graphics::{project_screen_to_world, LocalUniforms, to_vec2};
use crate::dung_gen::{DungGen, WallDirection};
use rand::{thread_rng, Rng};
use std::time::SystemTime;

pub struct DunGenSystem {
    pub dungeon: DungGen,
}

impl<'a> System<'a> for DunGenSystem {
    type SystemData = ();

    fn run(&mut self, (): Self::SystemData) {}

    fn setup(&mut self, world: &mut World) {
        use crate::dung_gen::TileType;

        for (&(x, y), &wall_type) in self.dungeon.world.iter() {
            let pos = Vector2::new(x as f32, y as f32);
            let pos3d = Vector3::new(x as f32, y as f32, 0.0);

            let DARK_GRAY = Vector3::new(0.1, 0.1, 0.1);
            let LIGHT_GRAY = Vector3::new(0.2, 0.2, 0.2);

            match wall_type {
                TileType::Nothing => {
                    let model = {
                        let context = world.read_resource::<graphics::Context>();
                        StaticModel::new(
                            &context,
                            1,
                            Vector3::new(x as f32, y as f32, 1.0),
                            1.0,
                            0.0,
                            DARK_GRAY,
                        )
                    };
                    world
                        .create_entity()
                        // .with(Position(pos)) ?
                        .with(model)
                        .build();
                }
                TileType::Floor => {
                    let model = {
                        let context = world.read_resource::<graphics::Context>();
                        StaticModel::new(
                            &context,
                            1,
                            pos3d,
                            1.0,
                            0.0,
                            DARK_GRAY,
                        )
                    };
                    world
                        .create_entity()
                        .with(Position(pos))
                        .with(FloorTile)
                        .with(model)
                        .build();
                }
                TileType::Wall(maybe_direction) => {
                    let dir = match maybe_direction {
                        Some(WallDirection::South) => 180.0,
                        Some(WallDirection::East) => 270.0,
                        Some(WallDirection::West) => 90.0,
                        _ => 0.0,
                    };
                    let model = {
                        let context = world.read_resource::<graphics::Context>();
                        match maybe_direction {
                            None => StaticModel::new(
                                &context,
                                0,
                                pos3d,
                                1.0,
                                0.0,
                                LIGHT_GRAY,
                            ),
                            Some(_) => StaticModel::new(
                                &context,
                                3,
                                pos3d,
                                1.0,
                                dir,
                                DARK_GRAY,
                            ),
                        }
                    };
                    world
                        .create_entity()
                        .with(Position(pos))
                        .with(WallTile)
                        .with(model)
                        .with(Orientation(dir))
                        .with(StaticBody)
                        .with(SquareCollider { side_length: 1.0 })
                        .build();
                }
                TileType::LadderDown => {
                    let model = {
                        let context = world.read_resource::<graphics::Context>();
                        StaticModel::new(
                            &context,
                            5,
                            pos3d,
                            1.0,
                            0.0,
                            DARK_GRAY,
                        )
                    };
                    world
                        .create_entity()
                        .with(Position(pos))
                        .with(FloorTile)
                        .with(model)
                        .build();
                }
                TileType::LadderUp => (),
            }
        };
    }
}
