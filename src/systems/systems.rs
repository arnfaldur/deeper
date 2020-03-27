use specs::prelude::*;
use std::f32::consts::PI;
use std::ops::Mul;
use zerocopy::{AsBytes};

extern crate cgmath;
use cgmath::{ Vector2, Vector3 };

use crate::graphics;
use crate::components::components::*;

pub struct MovementSystem;

impl<'a> System<'a> for MovementSystem {
    type SystemData = (WriteStorage<'a, Position>, ReadStorage<'a, Velocity>);

    fn run(&mut self, (mut pos, vel): Self::SystemData) {
        for (pos, vel) in (&mut pos, &vel).join() {
            pos.0 += vel.0;
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

struct CameraSystem;

impl<'a> System<'a> for CameraSystem {
    type SystemData = (WriteStorage<'a, Camera>, ReadStorage<'a, Position3D>);
    fn run(&mut self, (camera, pos): Self::SystemData) {}
}

pub struct GraphicsSystem {
    pub model_array: Vec<graphics::Model>,
    pub sc_desc: wgpu::SwapChainDescriptor,
    pub swap_chain: wgpu::SwapChain,
    pub queue: wgpu::Queue,
}

impl GraphicsSystem {
    pub fn new(
        model_array: Vec<graphics::Model>,
        sc_desc: wgpu::SwapChainDescriptor,
        swap_chain: wgpu::SwapChain,
        queue: wgpu::Queue,
    ) -> Self {
        Self { model_array, sc_desc, swap_chain, queue }
    }
}

impl<'a> System<'a> for GraphicsSystem {
    type SystemData = (
        ReadExpect<'a, graphics::Context>,
        ReadExpect<'a, ActiveCamera>,
        ReadStorage<'a, Camera>,
        ReadStorage<'a, Target>,
        ReadStorage<'a, Position3D>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Model3D>,
    );

    fn run(&mut self, (context, active_cam, camera, target, pos3d, pos, models): Self::SystemData) {

        let frame = self.swap_chain.get_next_texture().unwrap();

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
            self.sc_desc.width as f32 / self.sc_desc.height as f32,
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
            &wgpu::CommandEncoderDescriptor{ todo: 0 }
        );

        encoder.copy_buffer_to_buffer(
            &new_uniform_buf,
            0,
            &context.uniform_buf,
            0,
            16 * 4
        );

        let mut uniforms = vec!();

        for (pos, model) in (&pos, &models).join() {
            use cgmath::SquareMatrix;
            use cgmath::Quaternion;
            use cgmath::Euler;
            let mut matrix = Matrix4::from_scale(model.scale);
            matrix = Matrix4::from_angle_z(cgmath::Deg(model.z_rotation)) * matrix;
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
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    load_op: wgpu::LoadOp::Clear,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: wgpu::Color { r: 0.1, g: 0.2, b: 0.3, a: 1.0 }
                }],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor{
                    attachment: &context.depth_view,
                    depth_load_op: wgpu::LoadOp::Clear,
                    depth_store_op: wgpu::StoreOp::Store,
                    clear_depth: 1.0,
                    stencil_load_op: wgpu::LoadOp::Clear,
                    stencil_store_op: wgpu::StoreOp::Store,
                    clear_stencil: 0
                }),
            });

            rpass.set_pipeline(&context.pipeline);
            rpass.set_bind_group(0, &context.bind_group, &[]);

            for (_, model) in (&pos, &models).join() {
                for mesh in &self.model_array[model.idx].meshes {
                    rpass.set_bind_group(1, &model.bind_group, &[]);
                    rpass.set_vertex_buffer(0, &mesh.vertex_buffer, 0, 0);
                    rpass.draw(0..mesh.num_vertices as u32, 0..1)
                }
            }
        }

        let command_buf = encoder.finish();

        self.queue.submit(&[command_buf]);

    }

    fn setup(&mut self, world: &mut World) {
    }
}

pub struct PlayerSystem {
    //               responsibility of the input handling system exactly
    last_mouse_pos: Vector2<f32>,
}

impl PlayerSystem {
    pub fn new() -> Self { Self { last_mouse_pos: Vector2::new(0.0, 0.0) } }
}

// Note(Jökull): Is this really just the input handler?
impl<'a> System<'a> for PlayerSystem {
    type SystemData = (
        ReadExpect<'a, graphics::Context>,
        ReadExpect<'a, InputState>,
        ReadExpect<'a, Player>,
        ReadExpect<'a, PlayerCamera>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Position3D>,
        ReadStorage<'a, Camera>,
        WriteStorage<'a, Model3D>,
        WriteStorage<'a, Velocity>,
        WriteStorage<'a, SphericalOffset>,
    );

    fn run(&mut self, (context, input, player, player_cam, pos, pos3d, cam, mut model, mut vel, mut offset): Self::SystemData) {
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

        let mut player_vel = vel.get_mut(player.entity).unwrap();
        let player_pos = pos.get(player.entity).unwrap();
        player_vel.0 = Vector2::new(0.0, 0.0);

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
            let mx_correction = cgmath::Matrix4::new(
                1.0, 0.0, 0.0, 0.0,
                0.0, -1.0, 0.0, 0.0,
                0.0, 0.0, 0.5, 0.0,
                0.0, 0.0, 0.5, 1.0,
            );

            if let Some(mouse_world_pos) = project_screen_to_world(
                Vector3::new(mouse_pos.x, 1080.0 - mouse_pos.y, 1.0),
                mx_correction * mx_projection * mx_view,
                Vector4::new(0,0,1920,1080),
            ) {

                let ray_delta = mouse_world_pos - camera_pos.0;
                let t = mouse_world_pos.z / ray_delta.z;
                let ray_hit = mouse_world_pos - ray_delta * t;

                let difference = (ray_hit - player_pos.to_vec3());

                let difference = difference * (1.0 / graphics::length(difference));
                player_vel.0.x = difference.x * player.speed * 10.0;
                player_vel.0.y = difference.y * player.speed * 10.0;

                let model = model.get_mut(player.entity).unwrap();
                let mut new_rotation = (difference.y / difference.x).atan() / PI * 180.0;
                if difference.x > 0.0 {
                    new_rotation += 180.0;
                }
                model.z_rotation = new_rotation;
            }
        }
    }

    fn setup(&mut self, world: &mut World) {
        println!("PlayerSystem setup!");
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

        //for _ in (&dynamics, &statics).join() {
        //    println!("There's a naughty static body that really feels dynamic inside.");
        //    exit(0);
        //}

        //for (_, pos_d, vel, circle_d) in (&dynamics, &pos, &mut vel, &circles).join() {
        //    for (_, pos_s, circle_s) in (&statics, &pos, &circles).join() {
        //        if nalgebra::distance((pos_d.0 + vel.0), (pos_s.0)) < (circle_d.radius + circle_s.radius) {

        //            let diff = pos_s.0 - pos_d.0;
        //            let collinear_part = diff.scale_by(vel.0.dot(diff));
        //            vel.0 -= collinear_part;
        //        }
        //    }
        //}

        // let boi: Entity;
        // let igi: Position;
        // for (entity_a, pos_a, stat, circle_a) in (&ents, &pos, &statics, &circles).join() {
        //     for (entity_b, pos_b, dynamic, circle_b) in (&ents, &pos, &dynamics, &circles).join() {
        //         if entity_a != entity_b && (pos_a.0.distance_to(pos_b.0) < (circle_a.radius + circle_b.radius)) {
        //             println!("{:?} is colliding with {:?}!", entity_a, entity_b);
        //         }
        //     }
        // }
    }
}

use crate::dung_gen::DungGen;
use std::process::exit;
use self::cgmath::{Matrix4, Vector4};
use crate::input::InputState;
use crate::graphics::{project_screen_to_world, LocalUniforms};

pub struct DunGenSystem {
    pub dungeon: DungGen,
}

impl<'a> System<'a> for DunGenSystem {
    type SystemData = ();

    fn run(&mut self, (): Self::SystemData) {}

    fn setup(&mut self, world: &mut World) {
        use crate::dung_gen::WallType;

        for x in 0..=self.dungeon.width {
            for y in 0..=self.dungeon.height {
                match self.dungeon.world.get(&(x, y)) {
                    None => (),
                    Some(&value) => {

                        match value {
                            WallType::FLOOR => {
                                let model = {
                                    let context = world.read_resource::<graphics::Context>();
                                    Model3D::from_index(&context, 0).with_tint(Vector3::new(0.2, 0.2, 0.2))
                                };
                                let mut ent = world.create_entity()
                                    .with(Position(Vector2::new(x as f32, y as f32)))
                                    .with(FloorTile)
                                    .with(model).build(); //.with_tint(Color::DARKGRAY))
                            },
                            WallType::WALL => {
                                let model = {
                                    let context = world.read_resource::<graphics::Context>();
                                    Model3D::from_index(&context, 1).with_tint(Vector3::new(0.5, 0.5, 0.5))
                                };
                                let mut ent = world.create_entity()
                                    .with(Position(Vector2::new(x as f32, y as f32)))
                                    .with(WallTile)
                                    .with(StaticBody)
                                    .with(CircleCollider { radius: 0.5 })
                                    .with(model).build();
                            },
                            _ => (),
                            //WallType::WALL_NORTH => {
                            //    ent.with(WallTile)
                            //        //.with(Model3D::from_index(0)
                            //        //    //.with_tint(Color::DARKGRAY)
                            //        //    .with_z_rotation(0.0)
                            //        //)
                            //        .with(StaticBody)
                            //        .with(CircleCollider { radius: 0.5 })
                            //}
                            //WallType::WALL_SOUTH => {
                            //    ent.with(WallTile)
                            //        //.with(Model3D::from_index(0)
                            //        //    //.with_tint(Color::DARKGRAY)
                            //        //    .with_z_rotation(180.0)
                            //        //)
                            //        .with(StaticBody)
                            //        .with(CircleCollider { radius: 0.5 })
                            //}
                            //WallType::WALL_EAST => {
                            //    ent.with(WallTile)
                            //        //.with(Model3D::from_index(0)
                            //        //    //.with_tint(Color::DARKGRAY)
                            //        //    .with_z_rotation(-90.0)
                            //        //)
                            //        .with(StaticBody)
                            //        .with(CircleCollider { radius: 0.5 })
                            //}
                            //WallType::WALL_WEST => {
                            //    ent.with(WallTile)
                            //        //.with(Model3D::from_index(0)
                            //        //    //.with_tint(Color::DARKGRAY)
                            //        //    .with_z_rotation(90.0)
                            //        //)
                            //        .with(StaticBody)
                            //        .with(CircleCollider { radius: 0.5 })
                            //}
                            //WallType::NOTHING => {
                            //    ent
                            //}
                            //WallType::DEBUG => {
                            //    ent
                            //}
                        }
                    }
                }
            }
        }
    }
}

