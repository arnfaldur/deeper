use specs::prelude::*;
use std::f32::consts::PI;
use std::ops::Mul;

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
    pub context : graphics::Context,
    pub model_array: Vec<graphics::Model>,
    pub sc_desc: wgpu::SwapChainDescriptor,
    pub swap_chain: wgpu::SwapChain,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

impl GraphicsSystem {
    pub fn new(
        context: graphics::Context,
        model_array: Vec<graphics::Model>,
        sc_desc: wgpu::SwapChainDescriptor,
        swap_chain: wgpu::SwapChain,
        device: wgpu::Device,
        queue: wgpu::Queue,
    ) -> Self {
        Self { context, model_array, sc_desc, swap_chain, device, queue }
    }
}

impl<'a> System<'a> for GraphicsSystem {
    type SystemData = (
        ReadExpect<'a, ActiveCamera>,
        ReadStorage<'a, Camera>,
        ReadStorage<'a, Target>,
        ReadStorage<'a, Position3D>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Model3D>,
    );

    fn run(&mut self, (active_cam, camera, target, pos3d, pos, models): Self::SystemData) {
        let frame = self.swap_chain.get_next_texture();

        let mx_total = graphics::generate_matrix(self.sc_desc.width as f32 / self.sc_desc.height as f32, 0.0);
        let mx_ref: &[f32; 16] = mx_total.as_ref();

        let new_uniform_buf = self.device.create_buffer_mapped::<f32>(
            16,
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_SRC,
        ).fill_from_slice(mx_ref.as_ref());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor{ todo: 0});

        encoder.copy_buffer_to_buffer(&new_uniform_buf, 0, &self.context.uniform_buf, 0, 16 * 4);

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    load_op: wgpu::LoadOp::Clear,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: wgpu::Color { r: 0.1, g: 0.2, b: 0.3, a: 1.0 }
                }],
                depth_stencil_attachment: None
            });

            rpass.set_pipeline(&self.context.pipeline);
            rpass.set_bind_group(0, &self.context.bind_group, &[]);

            for (_, model) in (&pos, &models).join() {
                for mesh in &self.model_array[model.idx].meshes {
                    rpass.set_vertex_buffers(0, &[(&mesh.vertex_buffer, 0)]);
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
        ReadExpect<'a, Player>,
        ReadExpect<'a, PlayerCamera>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Position3D>,
        ReadStorage<'a, Camera>,
        WriteStorage<'a, Model3D>,
        WriteStorage<'a, Velocity>,
        WriteStorage<'a, SphericalOffset>,
    );

    fn run(&mut self, (player, player_cam, pos, pos3d, cam, mut model, mut vel, mut offset): Self::SystemData) {
        //let camera = cam.get(player_cam.0).unwrap();
        //let camera_pos = pos3d.get(player_cam.0).unwrap();
        //let mut camera_offset = offset.get_mut(player_cam.0).unwrap();

        //let mouse_delta = rl.get_mouse_position() - self.last_mouse_pos;
        //self.last_mouse_pos = rl.get_mouse_position();

        //if rl.is_mouse_button_down(MOUSE_MIDDLE_BUTTON) {
        //    camera_offset.theta += camera_offset.theta_delta * mouse_delta.x;
        //    camera_offset.phi += camera_offset.phi_delta * mouse_delta.y;
        //    camera_offset.phi = camera_offset.phi.max(0.1 * PI).min(0.25 * PI);
        //}

        //let mut player_vel = vel.get_mut(player.entity).unwrap();
        //player_vel.0 = vec2(0.0, 0.0);

        //if rl.is_mouse_button_down(MOUSE_LEFT_BUTTON) {
        //    // Note(Jökull): We need a better solution for this
        //    let player_pos = pos.get(player.entity).unwrap();
        //    let rl_cam = raylib::camera::Camera::perspective(
        //        camera_pos.0,
        //        player_pos.to_vec3(),
        //        camera.up,
        //        camera.fov,
        //    );
        //    let mouse_ray = rl.get_mouse_ray(rl.get_mouse_position(), rl_cam);
        //    let t = mouse_ray.position.z / mouse_ray.direction.z;
        //    let ray_hit = mouse_ray.position - mouse_ray.direction.scale_by(t);
        //    let difference = (ray_hit - player_pos.to_vec3());
        //    let difference = difference.scale_by(1.0 / difference.length());
        //    player_vel.0.x = difference.x * player.speed;
        //    player_vel.0.y = difference.y * player.speed;

        //    let model = model.get_mut(player.entity).unwrap();
        //    let mut new_rotation = (difference.y / difference.x).atan() / PI * 180.0;
        //    if difference.x > 0.0 {
        //        new_rotation += 180.0;
        //    }
        //    model.z_rotation = new_rotation;
        //}
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
                        let mut ent = world.create_entity()
                            .with(Position(Vector2::new(x as f32, y as f32)));

                        match value {
                            WallType::FLOOR => {
                                ent.with(FloorTile)
                                    //.with(Model3D::from_index(1).with_tint(Color::DARKGRAY))
                            }
                            WallType::WALL => {
                                ent.with(WallTile)
                                    //.with(Model3D::from_index(0).with_tint(Color::LIGHTGRAY))
                                    .with(StaticBody)
                                    .with(CircleCollider { radius: 0.5 })
                            }
                            WallType::WALL_NORTH => {
                                ent.with(WallTile)
                                    .with(Model3D::from_index(0)
                                        //.with_tint(Color::DARKGRAY)
                                        .with_z_rotation(0.0)
                                    )
                                    .with(StaticBody)
                                    .with(CircleCollider { radius: 0.5 })
                            }
                            WallType::WALL_SOUTH => {
                                ent.with(WallTile)
                                    .with(Model3D::from_index(0)
                                        //.with_tint(Color::DARKGRAY)
                                        .with_z_rotation(180.0)
                                    )
                                    .with(StaticBody)
                                    .with(CircleCollider { radius: 0.5 })
                            }
                            WallType::WALL_EAST => {
                                ent.with(WallTile)
                                    .with(Model3D::from_index(0)
                                        //.with_tint(Color::DARKGRAY)
                                        .with_z_rotation(-90.0)
                                    )
                                    .with(StaticBody)
                                    .with(CircleCollider { radius: 0.5 })
                            }
                            WallType::WALL_WEST => {
                                ent.with(WallTile)
                                    .with(Model3D::from_index(0)
                                        //.with_tint(Color::DARKGRAY)
                                        .with_z_rotation(90.0)
                                    )
                                    .with(StaticBody)
                                    .with(CircleCollider { radius: 0.5 })
                            }
                            WallType::NOTHING => {
                                ent
                            }
                            WallType::DEBUG => {
                                ent
                            }
                        }.build();
                    }
                }
            }
        }
    }
}

