use specs::prelude::*;
use std::f32::consts::{PI, FRAC_PI_2};
use std::ops::{Mul, Deref, DerefMut};
use zerocopy::AsBytes;

extern crate cgmath;

use cgmath::{prelude::*, Vector2, Vector3};

use crate::{graphics, components};
use crate::loader;
use crate::input::{InputState, Key};
use crate::components::*;

pub struct MovementSystem;

impl<'a> System<'a> for MovementSystem {
    type SystemData = (
        ReadExpect<'a, FrameTime>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Velocity>,
    );

    fn run(&mut self, (frame_time, mut pos, vel): Self::SystemData) {
        for (pos, vel) in (&mut pos, &vel).join() {
            pos.0 += vel.0 * frame_time.0;
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

use notify::{RecommendedWatcher, DebouncedEvent, Watcher, RecursiveMode};
use std::sync::mpsc::{Sender, Receiver, channel};

pub struct HotLoaderSystem {
    pub watcher: RecommendedWatcher,
    pub rx: Receiver<DebouncedEvent>,
}

impl HotLoaderSystem {
    pub fn new() -> Self {
        let (tx, rx) = channel();

        let mut watcher: RecommendedWatcher =
            Watcher::new(tx, Duration::from_secs(2)).unwrap();
        watcher.watch("assets/Models/", RecursiveMode::Recursive);

        Self { rx, watcher }
    }
}

impl<'a> System<'a> for HotLoaderSystem {
    type SystemData = (
        WriteExpect<'a, loader::AssetManager>,
        ReadExpect<'a, InputState>,
        ReadExpect<'a, graphics::Context>,
    );

    fn run(&mut self, (mut ass_man, input, context): Self::SystemData) {
        if input.is_key_pressed(Key::L) {
            println!("Hotloading...");
            ass_man.load_models(&context);
        }
    }

    fn setup(&mut self, world: &mut World) {}
}

pub struct GraphicsSystem;

impl<'a> System<'a> for GraphicsSystem {
    type SystemData = (
        WriteExpect<'a, graphics::Context>,
        ReadExpect<'a, loader::AssetManager>,
        ReadExpect<'a, ActiveCamera>,
        ReadStorage<'a, Camera>,
        ReadStorage<'a, Target>,
        ReadStorage<'a, Position3D>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Orientation>,
        ReadStorage<'a, Model3D>,
        ReadStorage<'a, StaticModel>,
        ReadStorage<'a, HitPoints>,
    );

    fn run(&mut self, (mut context, ass_man, active_cam, camera, target, pos3d, pos, orient, models, static_model, hp): Self::SystemData) {
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

        for (pos, model, rotation, hp) in (&pos, &models, (&orient).maybe(), (&hp).maybe()).join() {
            let mut matrix = Matrix4::from_scale(model.scale);
            if let Some(rot) = rotation {
                matrix = Matrix4::from_angle_z(rot.0) * matrix;
            }
            matrix = Matrix4::from_translation(pos.to_vec3()) * matrix;

            let bloody_red = Vector4::unit_x() + Vector4::unit_w();

            let alb = Vector4::from(model.material.albedo);

            let mut redder_mat: graphics::Material = model.material.clone();
            if let Some(hp) = hp {
                redder_mat.albedo = bloody_red.lerp(alb, hp.health / hp.max).into();
            }

            let local_uniforms = graphics::LocalUniforms {
                model_matrix: matrix.into(),
                material: redder_mat,
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
                for mesh in &ass_man.models[model.idx].meshes {
                    rpass.set_vertex_buffer(0, &mesh.vertex_buffer, 0, 0);
                    rpass.draw(0..mesh.num_vertices as u32, 0..1)
                }
            }

            for (_, model) in (&pos, &models).join() {
                rpass.set_bind_group(1, &model.bind_group, &[]);
                for mesh in &ass_man.models[model.idx].meshes {
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
        Entities<'a>,
        Read<'a, LazyUpdate>,
        ReadExpect<'a, InputState>,
        ReadExpect<'a, graphics::Context>,
        ReadExpect<'a, Player>,
        ReadExpect<'a, PlayerCamera>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Position3D>,
        ReadStorage<'a, Camera>,
        ReadStorage<'a, Faction>,
        ReadStorage<'a, HitPoints>,
        WriteStorage<'a, Orientation>,
        WriteStorage<'a, Destination>,
        WriteStorage<'a, SphericalOffset>,
    );

    fn run(&mut self, (ents, updater, input, context, player, player_cam, pos, pos3d, cam, faction, hp, mut orient, mut dest, mut offset): Self::SystemData) {
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
                Vector3::new(mouse_pos.x, context.sc_desc.height as f32 - mouse_pos.y, 1.0),
                graphics::correction_matrix() * mx_projection * mx_view,
                Vector4::new(0, 0, context.sc_desc.width as i32, context.sc_desc.height as i32),
            ) {
                let ray_delta: Vector3<f32> = mouse_world_pos - camera_pos.0;
                let t: f32 = mouse_world_pos.z / ray_delta.z;
                let ray_hit = to_vec2(mouse_world_pos - ray_delta * t);

                dest.insert(player.entity, Destination(ray_hit));

                let difference: Vector2<f32> = (ray_hit - player_pos.0).normalize();

                let mut new_rotation = (difference.y / difference.x).atan() / PI * 180.0;
                if difference.x > 0.0 {
                    new_rotation += 180.0;
                }
                (player_orient.0).0 = new_rotation;
            }
        }
        if input.is_key_pressed(Key::Space) {
            for (ent, pos, &HitPoints { max, health }, &faction) in (&ents, &pos, &hp, &faction).join() {
                if faction == Faction::Enemies && pos.0.distance(player_pos.0) < 2.0
                    && cgmath::Basis2::<f32>::from_angle(player_orient.0).rotate_vector(-Vector2::unit_x()).dot(pos.0 - player_pos.0) > 0.0 {
                    updater.insert(ent, HitPoints { max, health: (health - 1.0).max(0.0) });
                }
            }
        }
    }
}

pub struct HitPointRegenSystem;
impl<'a> System<'a> for HitPointRegenSystem {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, FrameTime>,
        WriteStorage<'a, HitPoints>,
        Read<'a, LazyUpdate>,
    );

    fn run(&mut self, (ents, frame_time, mut hp, updater): Self::SystemData) {
        for (ent, hp) in (&ents, &mut hp).join() {
            if hp.health <= 0.0 {
                updater.remove::<AIFollow>(ent);
                updater.remove::<Destination>(ent);
            } else {
                hp.health += 1.337 * frame_time.0;
                hp.health = hp.max.min(hp.health);
            }
        }
    }
}

pub struct AIFollowSystem;

impl<'a> System<'a> for AIFollowSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Destination>,
        WriteStorage<'a, Orientation>,
        ReadStorage<'a, AIFollow>,
        ReadStorage<'a, Position>,
    );

    fn run(&mut self, (ents, mut dest, mut orient, follow, pos): Self::SystemData) {
        for (ent, mut orient, follow, hunter) in (&ents, (&mut orient).maybe(), &follow, &pos).join() {
            if let Some(hunted) = pos.get(follow.target) {
                let difference: Vector2<f32> = hunted.0 - hunter.0;
                let distance = difference.magnitude();
                if distance > follow.minimum_distance {
                    dest.insert(ent, Destination(hunted.0));
                    if let Some(orientation) = orient {
                        orientation.0 = cgmath::Deg::from(-difference.angle(Vector2::unit_y()));
                    }
                }
            }
        }
    }
}

pub(crate) struct GoToDestinationSystem;

impl<'a> System<'a> for GoToDestinationSystem {
    type SystemData = (
        ReadExpect<'a, FrameTime>,
        ReadStorage<'a, Destination>,
        ReadStorage<'a, Position>,
        WriteStorage<'a, Velocity>,
        ReadStorage<'a, Speed>,
        ReadStorage<'a, Acceleration>,
    );

    fn run(&mut self, (frame_time, dest, pos, mut vel, speed, acc): Self::SystemData) {
        for (dest, hunter, vel, speed, accel) in (&dest, &pos, &mut vel, &speed, &acc).join() {
            let to_dest: Vector2<f32> = dest.0 - hunter.0;
            let direction = to_dest.normalize();
            let time_to_stop = speed.0 / accel.0;
            let slowdown = FRAC_PI_2.min(to_dest.magnitude() / time_to_stop * 0.5).sin();
            let target_velocity = direction * speed.0 * slowdown;
            let delta: Vector2<f32> = (target_velocity - vel.0);
            let velocity_change = (accel.0 * frame_time.0).min(delta.magnitude());

            if delta != Vector2::unit_x() * 0.0 {
                vel.0 += delta.normalize() * velocity_change;
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
                    let delta_a = pos_a.0 + vel.get(ent_a).unwrap().0 * frame_time.0;
                    let delta_b = pos_b.0 + vel.get(ent_b).unwrap().0 * frame_time.0;
                    // vector from ent_a to ent_b
                    let position_delta = delta_a - delta_b;
                    // how much are we colliding?
                    let collision_depth = collision_distance - position_delta.magnitude();
                    // same as position_delta but without velocity applied
                    let collision_direction = position_delta.normalize();
                    if collision_depth > 0.0 {
                        // get_mut is necessary to appease the borrow checker
                        //vel.get_mut(ent_a).unwrap().0 += (position_delta.normalize_to(collision_depth));
                        //vel.get_mut(ent_b).unwrap().0 -= (position_delta.normalize_to(collision_depth));

                        vel.get_mut(ent_a).unwrap().0 += collision_direction * collision_depth / 2.0 / frame_time.0;
                        vel.get_mut(ent_b).unwrap().0 -= collision_direction * collision_depth / 2.0 / frame_time.0;
                    }
                }
            }
        }
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
                    vel_a.0 += (position_delta.normalize_to(collision_depth));
                }
            }
        }
        for (_, pos_a, vel_a, circle_a) in (&dynamics, &pos, &mut vel, &circles).join() {
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
        }
    }
}

pub struct MapSwitchingSystem;

impl<'a> System<'a> for MapSwitchingSystem {
    type SystemData = (
        ReadExpect<'a, Player>,
        WriteExpect<'a, MapTransition>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, MapSwitcher>,
    );

    fn run(&mut self, (player, mut trans, pos, switcher): Self::SystemData) {
        let player_pos = pos.get(player.entity).expect("The player has no position, can't run MapSwitchingSystem");
        for (pos, switcher) in (&pos, &switcher).join() {
            if pos.0.distance(player_pos.0) < 0.5 {
                *trans = switcher.0;
            }
        }
    }
}

use self::cgmath::{Matrix4, Vector4, Deg};
use crate::graphics::{project_screen_to_world, LocalUniforms, to_vec2};
use crate::dung_gen::DungGen;
use rand::{thread_rng, Rng};
use std::time::{SystemTime, Duration};
use futures::SinkExt;
use rand::prelude::*;

pub struct DunGenSystem;

impl<'a> System<'a> for DunGenSystem {
    type SystemData = (
        WriteExpect<'a, MapTransition>,
        ReadExpect<'a, graphics::Context>,
        ReadExpect<'a, loader::AssetManager>,
        ReadExpect<'a, Player>,
        WriteExpect<'a, i64>,
        Entities<'a>,
        ReadStorage<'a, TileType>,
        ReadStorage<'a, Faction>,
        Read<'a, LazyUpdate>,
    );

    fn run(&mut self, (mut trans, context, ass_man, player, mut floor, mut ents, tile, factions, updater): Self::SystemData) {
        match *trans {
            MapTransition::Deeper => {
                for (ent, _) in (&ents, &tile).join() {
                    ents.delete(ent);
                }
                for (ent, faction) in (&ents, &factions).join() {
                    if let Faction::Enemies = faction {
                        ents.delete(ent);
                    }
                }
                *floor += 1;
                println!("You have reached floor {}", *floor);
                let dungeon = DungGen::new()
                    .width(60)
                    .height(60)
                    .n_rooms(10)
                    .room_min(5)
                    .room_range(5)
                    .generate();

                let mut rng = thread_rng();
                let player_start = {
                    let (x, y) = dungeon
                        .room_centers
                        .choose(&mut rand::thread_rng()).unwrap()
                        .clone();
                    (x + rng.gen_range(-2, 2), y + rng.gen_range(-2, 2))
                };


                // Reset player position and stuff
                updater.insert(player.entity, Position(Vector2::new(player_start.0 as f32, player_start.1 as f32)));
                updater.insert(player.entity, Orientation(Deg(0.0)));
                updater.insert(player.entity, Velocity::new());

                let mut init_encoder = context.device.create_command_encoder(
                    &wgpu::CommandEncoderDescriptor { todo: 0 }
                );

                let mut lights: graphics::Lights = Default::default();

                lights.directional_light = graphics::DirectionalLight {
                    direction: [1.0, 0.8, 0.8, 0.0],
                    ambient: [0.01, 0.015, 0.02, 1.0],
                    color: [0.1, 0.1, 0.2, 1.0],
                };

                for (i, &(x, y)) in dungeon.room_centers.iter().enumerate() {
                    if i >= graphics::MAX_NR_OF_POINT_LIGHTS { break; }
                    lights.point_lights[i] = Default::default();
                    lights.point_lights[i].radius = 30.0;
                    lights.point_lights[i].position = [x as f32, y as f32, 5.0, 1.0];
                    lights.point_lights[i].color = [1.0, 0.4, 0.1, 1.0];
                }

                let temp_buf = context.device.create_buffer_with_data(
                    lights.as_bytes(),
                    wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_SRC,
                );

                init_encoder.copy_buffer_to_buffer(
                    &temp_buf,
                    0,
                    &context.lights_buf,
                    0,
                    std::mem::size_of::<graphics::Lights>() as u64,
                );

                let command_buffer = init_encoder.finish();

                context.queue.submit(&[command_buffer]);
                // End graphics shit

                for (&(x, y), &wall_type) in dungeon.world.iter() {
                    let pos = Vector2::new(x as f32, y as f32);
                    let pos3d = Vector3::new(x as f32, y as f32, 0.0);

                    let (cube_idx, plane_idx, wall_idx, stairs_down_idx, floor_idx) = {
                        (
                            ass_man.get_model_index("cube.obj").unwrap(),
                            ass_man.get_model_index("plane.obj").unwrap(),
                            ass_man.get_model_index("Wall.obj").unwrap(),
                            ass_man.get_model_index("StairsDown.obj").unwrap(),
                            ass_man.get_model_index("floortile.obj").unwrap(),
                        )
                    };
                    let mut entity = ents.create();
                    let model = StaticModel::new(
                        &context,
                        match wall_type {
                            TileType::Nothing => plane_idx,
                            TileType::Wall(None) => cube_idx,
                            TileType::Wall(Some(_)) => wall_idx,
                            TileType::Floor => floor_idx,
                            TileType::LadderDown => stairs_down_idx,
                        },
                        match wall_type {
                            TileType::Nothing => Vector3::new(x as f32, y as f32, 1.0),
                            _ => pos3d,
                        },
                        1.0,
                        match wall_type {
                            TileType::Wall(Some(WallDirection::South)) => 180.,
                            TileType::Wall(Some(WallDirection::East)) => 270.,
                            TileType::Wall(Some(WallDirection::West)) => 90.,
                            _ => 0.,
                        },
                        match wall_type {
                            TileType::Nothing => graphics::Material::dark_stone(),
                            _ => graphics::Material::darkest_stone(),
                        },
                    );
                    updater.insert(entity, model);
                    updater.insert(entity, wall_type);
                    match wall_type {
                        TileType::Nothing => {}
                        _ => {
                            updater.insert(entity, Position(pos));
                        }
                    }
                    match wall_type {
                        TileType::Wall(maybe_direction) => {
                            updater.insert(entity, StaticBody);
                            updater.insert(entity, SquareCollider { side_length: 1.0 });
                        }
                        TileType::LadderDown => {
                            updater.insert(entity, MapSwitcher(MapTransition::Deeper));
                        }
                        _ => {}
                    }
                    if TileType::Floor == wall_type && rng.gen_bool(((*floor - 1) as f64 * 0.05 + 1.).log2() as f64) {
                        let rad = rng.gen_range(0.1, 0.5);
                        let enemy = ents.create();
                        updater.insert(enemy, Position(pos));
                        updater.insert(enemy, Speed(rng.gen_range(1., 3.)));
                        updater.insert(enemy, Acceleration(rng.gen_range(3., 9.)));
                        updater.insert(enemy, Orientation(Deg(0.0)));
                        updater.insert(enemy, Velocity::new());
                        updater.insert(enemy, DynamicBody);
                        updater.insert(enemy, CircleCollider { radius: rad });
                        updater.insert(enemy, AIFollow {
                            target: player.entity,
                            minimum_distance: 2.0 + rad,
                        });
                        updater.insert(enemy, Faction::Enemies);
                        updater.insert(enemy, HitPoints { max: rng.gen_range(2.,5.), health: rng.gen_range(2.,5.) });
                        updater.insert(enemy,
                                       Model3D::from_index(&context, ass_man.get_model_index("monstroman.obj").unwrap())
                                           .with_material(graphics::Material::glossy(Vector3::<f32>::new(rng.gen(), rng.gen(), rng.gen())))
                                           .with_scale(rad * 1.7));
                    }
                };

                for _enemy in 0..128 {}
            }
            _ => {}
        }
        *trans = MapTransition::None;
    }
}
