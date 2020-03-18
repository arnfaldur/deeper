mod loader;

use loader::AssetManager;

mod dung_gen;

use dung_gen::{
    DungGen,
    TileKind,
};

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

fn main() {
    let mut ass_man = AssetManager::new();

    let ds = ass_man.load_display_settings();

    let (mut rl, thread) = raylib::init()
        .size(ds.screen_width, ds.screen_height)
        .resizable()
        .title("deeper")
        .build();

    rl.set_target_fps(ds.fps);

    use specs::{World, WorldExt, Builder};

    let mut world = World::new();

    register_components(&mut world);

    let model_array = vec![
        rl.load_model(&thread, "./assets/Models/cube.obj").unwrap(),
        rl.load_model(&thread, "./assets/Models/plane.obj").unwrap(),
    ];

    // Relinquish the raylib handle to the world
    world.insert(rl);

    // initialize dispacher with all game systems
    let mut dispatcher = DispatcherBuilder::new()
        .with(DunGenSystem, "DunGenSystem", &[])
        .with(MovementSystem, "MovementSystem", &[])
        .with(PlayerSystem, "PlayerSystem", &[])
        .with(SphericalFollowSystem, "SphericalFollowSystem", &["PlayerSystem", "MovementSystem"])
        .with_thread_local(GraphicsSystem { thread, model_array })
        .build();

    let player = world.create_entity()
        .with(Player)
        .with(Position::new())
        .with(Model3D::from_index(0))
        .build();

    let active_camera = world.create_entity()
        .with(components::Camera {
            up: Vector3::new(0.0, 0.0, 1.0),
            fov: 40.0,
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
        .with(Model3D::from_index(1))
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