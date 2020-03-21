use specs::prelude::*;
use raylib::prelude::*;
use std::f32::consts::PI;
use std::ops::{Mul};

use crate::components::components::*;

pub struct MovementSystem;

impl<'a> System<'a> for MovementSystem {
    type SystemData = (WriteStorage<'a, Position>, ReadStorage<'a, Velocity>);

    fn run(&mut self, (mut pos, vel): Self::SystemData) {
        for (pos, vel) in (&mut pos, &vel).join() {
            pos.x += vel.x;
            pos.y += vel.y;
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
    type SystemData = (WriteStorage<'a, crate::components::components::Camera>, ReadStorage<'a, Position3D>);

    fn run(&mut self, (camera, pos): Self::SystemData) {
        for (camera, pos) in (&camera, &pos).join() {}
    }
}

extern crate raylib;
use raylib::shaders::Shader;

pub struct GraphicsSystem {
    pub thread: RaylibThread,
    pub model_array: Vec<Model>,
    pub l_shader: Shader,
    mat_model_loc: i32,
    eye_position_loc: i32,
}

impl GraphicsSystem {
    pub fn new(thread: RaylibThread, model_array: Vec<Model>, l_shader: Shader) -> Self { Self { thread, model_array, l_shader, mat_model_loc: 0, eye_position_loc: 0 } }
}

impl<'a> System<'a> for GraphicsSystem {
    type SystemData = (
        WriteExpect<'a, RaylibHandle>,
        ReadExpect<'a, ActiveCamera>,
        ReadStorage<'a, Player>,
        ReadStorage<'a, crate::components::components::Camera>,
        ReadStorage<'a, Target>,
        ReadStorage<'a, Position3D>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Model3D>,
    );

    fn run(&mut self, (mut rl, active_cam, player, camera, target, pos3d, pos, models): Self::SystemData) {
        let fps = 1.0 / rl.get_frame_time();
        let mut d2: RaylibDrawHandle = rl.begin_drawing(&self.thread);

        d2.clear_background(Color::BLACK);

        {
            let active_camera = camera.get(active_cam.0).unwrap();
            let active_target = target.get(active_cam.0).unwrap();
            let camera_position = pos3d.get(active_cam.0).unwrap().0;

            self.l_shader.set_shader_value(self.eye_position_loc, camera_position);

            let mut d3 = d2.begin_mode_3D(
                Camera3D::perspective(
                    camera_position,
                    pos.get(active_target.0).unwrap().to_vec3(),
                    active_camera.up,
                    active_camera.fov,
                )
            );

            for (pos, model) in (&pos, &models).join() {
                let model_pos = pos.clone().to_vec3() + model.offset;
                self.l_shader.set_shader_value_matrix(
                    self.mat_model_loc,
                    Matrix::scale(model.scale, model.scale, model.scale).mul(Matrix::translate(model_pos.x, model_pos.y, model_pos.z)),
                );
                d3.draw_model_ex(
                    &self.model_array[model.idx],
                    model_pos,
                    Vector3::new(0.0, 0.0, 1.0),
                    model.z_rotation,
                    Vector3::new(model.scale, model.scale, model.scale),
                    model.tint
                );
            }
        }

        d2.draw_text("deeper", 12, 12, 30, Color::WHITE);
        d2.draw_text(&format!("FPS {}", fps), 12, 46, 18, Color::WHITE);
    }

    fn setup(&mut self, world: &mut World) {
        self.mat_model_loc = self.l_shader.get_shader_location("matModel");
        self.eye_position_loc = self.l_shader.get_shader_location("eyePosition");

        println!("GraphicsSystem setup!");
    }
}

pub struct PlayerSystem;

impl<'a> System<'a> for PlayerSystem {
    type SystemData = (
        ReadExpect<'a, RaylibHandle>,
        ReadStorage<'a, Player>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, crate::components::components::Camera>,
        ReadStorage<'a, Target>,
        WriteStorage<'a, SphericalOffset>,
    );

    fn run(&mut self, (rl, player, mut pos, mut cam, target, mut offset): Self::SystemData) {
        use raylib::consts::KeyboardKey::*;
        let cam_speed = 0.1;
        for (_, pos) in (&player, &mut pos).join() {
            if rl.is_key_down(KEY_UP) { pos.y += cam_speed }
            if rl.is_key_down(KEY_DOWN) { pos.y -= cam_speed }
            if rl.is_key_down(KEY_LEFT) { pos.x -= cam_speed }
            if rl.is_key_down(KEY_RIGHT) { pos.x += cam_speed }
        }

        let ang_vel = 0.03;
        for (cam, target, offset) in (&mut cam, &target, &mut offset).join() {
            if let Some(_player) = player.get(target.0) {
                if rl.is_key_down(KEY_E) { offset.phi += ang_vel; }
                if rl.is_key_down(KEY_D) { offset.phi -= ang_vel; }
                offset.phi = offset.phi.max(0.3).min(PI / 2.0 - 0.3);

                if rl.is_key_down(KEY_S) { offset.theta -= ang_vel; }
                if rl.is_key_down(KEY_F) { offset.theta += ang_vel; }

                if rl.is_key_down(KEY_W) { offset.radius -= cam_speed; }
                if rl.is_key_down(KEY_R) { offset.radius += cam_speed; }
                offset.radius = offset.radius.max(2.0).min(10.0);

                cam.fov += rl.get_mouse_wheel_move() as f32;
            };
        }
    }

    fn setup(&mut self, world: &mut World) {
        println!("PlayerSystem setup!");
    }
}

use crate::dung_gen::{DungGen};

pub struct DunGenSystem {
    pub dungeon : DungGen,
}

impl<'a> System<'a> for DunGenSystem {
    type SystemData = ();

    fn run(&mut self, (): Self::SystemData) {

    }

    fn setup(&mut self, world: &mut World) {
        use crate::dung_gen::{FLOOR, WALL, WALL_NORTH, WALL_SOUTH, WALL_EAST, WALL_WEST, NOTHING};

        for x in 0..=self.dungeon.width {
            for y in 0..=self.dungeon.height {
                match self.dungeon.world.get(&(x, y)) {
                    None => (),
                    Some(&value) => {
                        match value {
                            FLOOR => {
                                world.create_entity()
                                    .with(Position { x: x as f32, y: y as f32 })
                                    .with(FloorTile)
                                    .with(Model3D::from_index(1).with_tint(Color::DARKGRAY))
                                    .build();
                            },
                            WALL => {
                                world.create_entity()
                                    .with(Position { x: x as f32, y: y as f32 })
                                    .with(WallTile)
                                    .with(Model3D::from_index(0).with_tint(Color::LIGHTGRAY))
                                    .build();
                            },
                            WALL_NORTH => {
                                world.create_entity()
                                    .with(Position { x: x as f32, y: y as f32 })
                                    .with(WallTile)
                                    .with(Model3D::from_index(3)
                                        .with_tint(Color::DARKGRAY)
                                        .with_z_rotation(0.0)
                                    ).build();
                            },
                            WALL_SOUTH => {
                                world.create_entity()
                                    .with(Position { x: x as f32, y: y as f32 })
                                    .with(WallTile)
                                    .with(Model3D::from_index(3)
                                        .with_tint(Color::DARKGRAY)
                                        .with_z_rotation(180.0)
                                    ).build();
                            },
                            WALL_EAST => {
                                world.create_entity()
                                    .with(Position { x: x as f32, y: y as f32 })
                                    .with(WallTile)
                                    .with(Model3D::from_index(3)
                                        .with_tint(Color::DARKGRAY)
                                        .with_z_rotation(-90.0)
                                    ).build();
                            },
                            WALL_WEST => {
                                world.create_entity()
                                    .with(Position { x: x as f32, y: y as f32 })
                                    .with(WallTile)
                                    .with(Model3D::from_index(3)
                                        .with_tint(Color::DARKGRAY)
                                        .with_z_rotation(90.0)
                                    ).build();
                            },
                            // Note(Jökull): Way too slow
                            //NOTHING => {
                            //    world.create_entity()
                            //        .with(Position { x: x as f32, y: y as f32 })
                            //        .with(FloorTile)
                            //        .with(
                            //            Model3D::from_index(1)
                            //                .with_tint(Color::DARKGRAY)
                            //                .with_offset(Vector3::new(0.0, 0.0, 1.0))
                            //        ).build();
                            //},
                            _ => (),
                        }
                    }
                }
            }
        }
    }
}

