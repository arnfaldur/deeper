extern crate specs;

use specs::{
    DispatcherBuilder,
    WorldExt,
    Builder,
    System,
    AccessorCow,
    RunningTime,
    Component,
    VecStorage
};

use raylib::prelude::*;
use std::f32::consts::PI;
use rand::seq::SliceRandom;
use std::ops::{Add, Mul};
use std::process::exit;
use specs::prelude::*;

#[derive(Component, Debug, Copy, Clone)]
#[storage(VecStorage)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

impl Position {
    pub fn new() -> Position { Position { x : 0.0, y : 0.0 } }
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

impl Velocity {
    pub fn new() -> Velocity { Velocity { x : 0.0, y : 0.0 } }
}
#[derive(Component)]
#[storage(VecStorage)]
pub struct Agent;

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

impl From<&Position> for Vector3 {
    fn from(pos: &Position) -> Vector3 {
        Vector3::new(pos.x, pos.y, 0.0)
    }
}

impl Position {
    pub fn to_vec3(self) -> Vector3 {
        Vector3::new(self.x, self.y, 0.0)
    }
}

impl From<&Position> for Vector2 {
    fn from(pos: &Position) -> Vector2 {
        Vector2::new(pos.x, pos.y)
    }
}

impl From<&Velocity> for Vector2 {
    fn from(pos: &Velocity) -> Vector2 {
        Vector2::new(pos.x, pos.y)
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Camera {
    pub fov: f32,
    pub up: Vector3,
}

#[derive(Component)]
pub struct Target(pub Entity);

#[derive(Component)]
pub struct Position3D(pub Vector3);

#[derive(Component)]
pub struct ActiveCamera(pub Entity);

#[derive(Component)]
pub struct SphericalOffset {
    pub theta: f32,
    pub phi: f32,
    pub radius: f32,
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

    fn run(&mut self, (camera, pos): Self::SystemData) {
        for (camera, pos) in (&camera, &pos).join() {}
    }
}

#[derive(Component)]
pub struct Model3D {
    pub idx : usize,
}

impl Model3D {
    pub fn new() -> Model3D { Model3D { idx  : 0 } }
    pub fn from_index(index : usize) -> Model3D { Model3D { idx : index } }
}

pub struct GraphicsSystem {
    pub thread: RaylibThread,
    pub model_array: Vec<Model>,
}

impl<'a> System<'a> for GraphicsSystem {
    type SystemData = (
        WriteExpect<'a, RaylibHandle>,
        ReadExpect<'a, ActiveCamera>,
        ReadStorage<'a, Player>,
        ReadStorage<'a, Camera>,
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
            let mut d3 = d2.begin_mode_3D(
                Camera3D::perspective(
                    pos3d.get(active_cam.0).unwrap().0,
                    pos.get(active_target.0).unwrap().to_vec3(),
                    active_camera.up,
                    active_camera.fov,
                )
            );

            for (pos, model) in (&pos, &models).join() {
                d3.draw_model(&self.model_array[model.idx], pos.clone().to_vec3(), 1.0, Color::LIGHTGRAY);
            }
        }

        d2.draw_text("deeper", 12, 12, 30, Color::WHITE);
        d2.draw_text(&format!("FPS {}", fps), 12, 46, 18, Color::WHITE);
    }

    fn setup(&mut self, world: &mut World) {
        println!("GraphicsSystem setup!");
    }
}

pub struct PlayerSystem;

impl<'a> System<'a> for PlayerSystem {
    type SystemData = (
        ReadExpect<'a, RaylibHandle>,
        ReadStorage<'a, Player>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, Camera>,
        ReadStorage<'a, Target>,
        WriteStorage<'a, SphericalOffset>,
    );

    fn run(&mut self, (rl, player, mut pos, mut cam, target, mut offset): Self::SystemData) {
        use raylib::consts::KeyboardKey::*;
        let cam_speed = 0.1;
        for (_, pos) in (&player, &mut pos).join() {
            if rl.is_key_down(KEY_UP)    { pos.y += cam_speed }
            if rl.is_key_down(KEY_DOWN)  { pos.y -= cam_speed }
            if rl.is_key_down(KEY_LEFT)  { pos.x -= cam_speed }
            if rl.is_key_down(KEY_RIGHT) { pos.x += cam_speed }

            use raylib::consts::MouseButton::*;
        }

        let ang_vel = 0.03;
        for (cam, target, offset) in (&mut cam, &target, &mut offset).join() {
            if let Some(player) = player.get(target.0) {
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
