#![feature(nll)]

mod loader;

use loader::AssetManager;

mod dung_gen;

use dung_gen::{
    DungGen,
    TileKind,
};

mod old_main;
mod components;

use components::*;

use raylib::prelude::*;
use std::f32::consts::PI;
use rand::seq::SliceRandom;
use std::ops::{Add, Mul};
use std::process::exit;
use specs::prelude::*;
use specs::{DispatcherBuilder, WorldExt, Builder, System, AccessorCow, RunningTime};

use specs::Component;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Camera {
    fov: f32,
    up: Vector3,
}

#[derive(Component)]
struct Target(Entity);

#[derive(Component)]
struct Position3D(Vector3);

#[derive(Component)]
struct ActiveCamera(Entity);

#[derive(Component)]
struct SphericalOffset {
    theta: f32,
    phi: f32,
    radius: f32,
}

struct SphericalFollowSystem;

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
            pos3d.0.y += follow.radius * follow.phi.sin();
            pos3d.0.z += follow.radius * follow.theta.sin() * follow.phi.cos();
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
struct Model3D(usize);

struct GraphicsSystem {
    thread: RaylibThread,
    model_array: Vec<Model>,
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
                d3.draw_model(&self.model_array[model.0], pos.clone().to_vec3(), 1.0, Color::LIGHTGRAY);
            }
        }

        d2.draw_text("deeper", 12, 12, 30, Color::WHITE);
        d2.draw_text(&format!("FPS {}", fps), 12, 46, 18, Color::WHITE);
    }

    fn setup(&mut self, world: &mut World) {
        println!("GraphicsSystem setup!");
    }
}

struct PlayerSystem;

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
            if rl.is_key_down(KEY_UP) { pos.y += cam_speed }
            if rl.is_key_down(KEY_DOWN) { pos.y -= cam_speed }
            if rl.is_key_down(KEY_LEFT) { pos.x -= cam_speed }
            if rl.is_key_down(KEY_RIGHT) { pos.x += cam_speed }
        }

        let ang_vel = 0.03;
        for (cam, target, offset) in (&mut cam, &target, &mut offset).join() {
            if let Some(player) = player.get(target.0) {
                if rl.is_key_down(KEY_E) { offset.phi += ang_vel; }
                if rl.is_key_down(KEY_D) { offset.phi -= ang_vel; }
                offset.phi = offset.phi.max(0.3).min(PI / 2.0 - 0.3);

                if rl.is_key_down(KEY_S) { offset.theta += ang_vel; }
                if rl.is_key_down(KEY_F) { offset.theta -= ang_vel; }

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

fn main() {
    let mut ass_man = AssetManager::new();

    let ds = ass_man.load_display_settings();

    let (mut rl, thread) = raylib::init()
        .size(ds.screen_width, ds.screen_height)
        .resizable()
        .title("deeper")
        .build();

    //rl.set_target_fps(ds.fps);

    use specs::{World, WorldExt, Builder};

    let mut world = World::new();

    world.register::<Position>();
    world.register::<Position3D>();
    world.register::<Velocity>();
    world.register::<Player>();
    world.register::<Camera>();
    world.register::<Target>();
    world.register::<ActiveCamera>();
    world.register::<SphericalOffset>();
    world.register::<Model3D>();

    let model_array = vec![
        rl.load_model(&thread, "./assets/Models/cube.obj").unwrap(),
        rl.load_model(&thread, "./assets/Models/plane.obj").unwrap(),
    ];

    // Relinquish the raylib handle to the world
    world.insert(rl);

    // initialize dispacher with all game systems
    let mut dispatcher = DispatcherBuilder::new()
        .with(MovementSystem, "MovementSystem", &[])
        .with(PlayerSystem, "PlayerSystem", &[])
        .with(SphericalFollowSystem, "SphericalFollowSystem", &["PlayerSystem", "MovementSystem"])
        .with_thread_local(GraphicsSystem { thread, model_array })
        .build();

    let player = world.create_entity()
        .with(Player)
        .with(Position { x: 0.0, y: 0.0 })
        .with(Model3D(0))
        .build();
    let active_camera = world.create_entity()
        .with(Camera {
            up: Vector3::up(),
            fov: 70.0,
        })
        .with(Target(player))
        .with(Position3D(vec3(0.0, 0.0, 0.0)))
        .with(SphericalOffset {
            theta: PI / 3.0,
            phi: PI / 4.0,
            radius: 4.5,
        })
        .build();
    world.insert(ActiveCamera(active_camera));

    world.create_entity()
        .with(Position { x: 1.0, y: 2.0 })
        .with(Model3D(1))
        .build();

    // Setup world
    dispatcher.setup(&mut world);
    // Main game loop
    while !window_should_close(&world) {
        // Should be the only thing in the loop, before the loop is completely removed
        dispatcher.dispatch(&mut world);
    }
}

fn window_should_close(world: &World) -> bool {
    let rl = world.read_resource::<RaylibHandle>();
    return rl.window_should_close();
}