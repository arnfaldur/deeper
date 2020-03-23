mod loader;

use loader::AssetManager;

mod dung_gen;

use dung_gen::DungGen;

mod components;
mod systems;

use crate::components::components::*;
use crate::systems::systems::*;

use rand::seq::SliceRandom;
use raylib::prelude::*;
use specs::prelude::*;

const FRAG_SRC: &str = include_str!("../shaders/test.frag");
const VERT_SRC: &str = include_str!("../shaders/test.vert");

fn main() {
    let mut ass_man = AssetManager::new();

    let dungeon = DungGen::new()
        .width(60)
        .height(60)
        .n_rooms(10)
        .room_min(5)
        .room_range(5)
        .generate();

    let player_start = dungeon
        .room_centers
        .choose(&mut rand::thread_rng())
        .unwrap()
        .clone();

    let ds = ass_man.load_display_settings();

    let (mut rl, thread) = raylib::init()
        .size(ds.screen_width, ds.screen_height)
        .resizable()
        .title("deeper")
        .build();

    rl.set_target_fps(ds.fps);

    let mut world = World::new();

    register_components(&mut world);

    let mut l_shader = rl.load_shader_code(&thread, Some(VERT_SRC), Some(FRAG_SRC));

    for i in 0..dungeon.room_centers.len() {
        let center = dungeon.room_centers[i];
        let prefix = format!("uPointLights[{}]", i);
        let is_lit_loc = l_shader.get_shader_location(&format!("{}.is_lit", prefix));
        let radius_loc = l_shader.get_shader_location(&format!("{}.radius", prefix));
        let position_loc = l_shader.get_shader_location(&format!("{}.position", prefix));
        let color_loc = l_shader.get_shader_location(&format!("{}.color", prefix));

        l_shader.set_shader_value(is_lit_loc, 1);
        l_shader.set_shader_value(radius_loc, 1000.0);
        l_shader.set_shader_value(position_loc, vec3(center.0 as f32, center.1 as f32, 1.0));
        let color = Vector4::new(0.9, 0.4, 0.1, 1.0);
        l_shader.set_shader_value(color_loc, color);
    }

    let mut model_array = vec![
        rl.load_model(&thread, "./assets/Models/cube.obj").unwrap(),
        rl.load_model(&thread, "./assets/Models/plane.obj").unwrap(),
        rl.load_model(&thread, "./assets/Models/Arissa/arissa.obj")
            .unwrap(),
        rl.load_model(
            &thread,
            "./assets/Models/DungeonCollection2/struct_large_straight_wall.obj",
        )
            .unwrap(),
        rl.load_model(&thread, "./assets/Models/walltest.obj")
            .unwrap(),
    ];

    for model in &mut model_array {
        let materials = model.materials_mut();
        let material = &mut materials[0];

        material.shader = *l_shader.as_ref();
    }

    // Relinquish the raylib handle to the world
    world.insert(rl);

    // initialize dispacher with all game systems
    let mut dispatcher = DispatcherBuilder::new()
        .with(DunGenSystem { dungeon }, "DunGenSystem", &[])
        .with(PlayerSystem::new(), "PlayerSystem", &[])
        .with(Physics2DSystem, "Physics2DSystem", &["PlayerSystem"])
        .with(
            MovementSystem,
            "MovementSystem",
            &["Physics2DSystem", "PlayerSystem"],
        )
        .with(
            SphericalFollowSystem,
            "SphericalFollowSystem",
            &["MovementSystem"],
        )
        .with_thread_local(GraphicsSystem::new(thread, model_array, l_shader))
        .build();

    let player = world
        .create_entity()
        .with(Position(vec2(player_start.0 as f32, player_start.1 as f32)))
        .with(DynamicBody)
        .with(CircleCollider { radius: 0.5 })
        .with(Velocity::new())
        .with(Model3D::from_index(2).with_scale(0.5))
        .build();

    let player_camera = world
        .create_entity()
        .with(components::components::Camera {
            up: Vector3::new(0.0, 0.0, 1.0),
            fov: 25.0,
        })
        .with(Target(player))
        .with(Position3D(vec3(0.0, 0.0, 0.0)))
        .with(SphericalOffset::new())
        .build();

    world.insert(Player::from_entity(player));
    world.insert(ActiveCamera(player_camera));
    world.insert(PlayerCamera(player_camera));

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
