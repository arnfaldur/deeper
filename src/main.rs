mod loader;
use loader::{
    AssetManager,
};

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

const frag_src: &str = include_str!("../shaders/test.frag");
const vert_src: &str = include_str!("../shaders/test.vert");

fn main() {
    let mut ass_man = AssetManager::new();

    let ds = ass_man.load_display_settings();

    let dungeon = DungGen::new()
        .width(75)
        .height(75)
        .n_rooms(10)
        .room_min(5)
        .room_range(15)
        .generate();

    let (mut rl, thread) = raylib::init()
        .size(ds.screen_width, ds.screen_height)
        .resizable()
        .title("deeper")
        .build();

    rl.set_target_fps(ds.fps);

    use specs::{World, WorldExt, Builder};

    let mut world = World::new();
    world.register::<Position>();
    world.register::<Velocity>();



    let mut last_mouse_pos = vec2(0.0, 0.0);

    let mut camera = Camera3D::perspective(
        vec3(0.0, 0.0, 0.0),
        vec3(0.0, 0.0, 0.0),
        Vector3::up(),
        70.0
    );

    let mut tht : f32 = PI / 3.0;
    let mut phi : f32 = PI / 4.0;
    let mut r   : f32= 4.5;

    let ang_vel = 0.01;

    let start_room = dungeon.room_centers.choose(&mut rand::thread_rng()).unwrap();
    let mut cam_pos = vec3(start_room.0 as f32, 0.0, start_room.1 as f32);

    let cam_speed = 0.1;

    let mut cube_model = rl.load_model(
        &thread,
        "./assets/Models/windmill.obj"
    ).unwrap();

    let materials = cube_model.materials_mut();
    let material = &mut materials[0];

    let mut shader = rl.load_shader_code(
        &thread,
        Some(vert_src),
        Some(frag_src)
    );

    let matModel_loc = shader.get_shader_location("matModel");
    let eyePosition_loc  = shader.get_shader_location("eyePosition");

    for i in 0..dungeon.room_centers.len() {
        let center = dungeon.room_centers[i];
        let prefix = format!("uPointLights[{}]", i);
        let is_lit_loc   = shader.get_shader_location(&format!("{}.is_lit", prefix));
        let radius_loc   = shader.get_shader_location(&format!("{}.radius", prefix));
        let position_loc = shader.get_shader_location(&format!("{}.position", prefix));
        let color_loc    = shader.get_shader_location(&format!("{}.color", prefix));

        shader.set_shader_value(is_lit_loc, 1);
        shader.set_shader_value(radius_loc, 10.0);
        shader.set_shader_value(position_loc, vec3(center.0 as f32, 1.0, center.1 as f32));
        let color = Vector4::new(1.0, 0.0, 0.0, 1.0);
        shader.set_shader_value(color_loc, color);
    }

    material.shader = *shader.as_ref();

    // Main game loop
    while !rl.window_should_close() {
        // Input handling
        let mouse_pos = rl.get_mouse_position();

        use raylib::consts::KeyboardKey::*;
        if rl.is_key_down(KEY_E) { phi += ang_vel; }
        if rl.is_key_down(KEY_D) { phi -= ang_vel; }
        phi = phi.max(0.3).min(PI / 2.0 - 0.3);

        if rl.is_key_down(KEY_S) { tht += ang_vel; }
        if rl.is_key_down(KEY_F) { tht -= ang_vel; }

        if rl.is_key_down(KEY_W) { r -= cam_speed; }
        if rl.is_key_down(KEY_R) { r += cam_speed; }
        r = r.max(2.0).min(10.0);

        if rl.is_key_down(KEY_UP)    {cam_pos.z += cam_speed}
        if rl.is_key_down(KEY_DOWN)  {cam_pos.z -= cam_speed}
        if rl.is_key_down(KEY_LEFT)  {cam_pos.x -= cam_speed}
        if rl.is_key_down(KEY_RIGHT) {cam_pos.x += cam_speed}

        //println!("tpr : ({} {} {}) fovy: {}, pos : {:?}", tht, phi, r, camera.fovy, cam_pos);

        camera.target = cam_pos;
        camera.position = cam_pos;
        camera.position.x += r * tht.cos() * phi.cos();
        camera.position.y += r * phi.sin();
        camera.position.z += r * tht.sin() * phi.cos();

        shader.set_shader_value(eyePosition_loc, camera.position);

        camera.fovy += rl.get_mouse_wheel_move() as f32;

        last_mouse_pos = mouse_pos;

        // Graphics
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::BLACK);

        d.draw_text("deeper", 12, 12, 30, Color::WHITE);

        // 3D graphics
        {
            let mut d2 = d.begin_mode_3D(camera);

            for x in 0..=dungeon.width {
                for y in 0..=dungeon.height {
                    match dungeon.world.get(&(x, y)) {
                        Some(&dung_gen::FLOOR) => {
                            let pos = vec3(x as f32, -0.5, y as f32);
                            shader.set_shader_value_matrix(
                                matModel_loc,
                                Matrix::translate(pos.x, pos.y, pos.z)
                            );
                            d2.draw_model(&cube_model, pos, 1.0, Color::DARKGRAY);
                        },
                        Some(&dung_gen::WALL) => {
                            let pos = vec3(x as f32, 0.5, y as f32);
                            shader.set_shader_value_matrix(
                                matModel_loc,
                                Matrix::translate(pos.x, pos.y, pos.z)
                            );
                            d2.draw_model(&cube_model, pos, 1.0, Color::LIGHTGRAY);
                        },
                        _ => (),
                    }
                }
            }
        }
    }
}
