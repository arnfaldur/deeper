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
use std::ops::Add;


fn main() {
    let mut ass_man = AssetManager::new();

    let ds = ass_man.load_display_settings();

    let dungeon = DungGen::new()
        .width(100)
        .height(100)
        .n_rooms(40)
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

    let floor_color = Color::new(50,50,50,255);
    let wall_color = Color::new(90,90,90,255);

    let mut sq_width : f32 = 0.5;

    let mut zoom = 10.0;

    let mut camera = Camera3D::orthographic(
        vec3(-5.0, zoom, -5.0),
        vec3(12.0, 0.0, 12.0),
        vec3(0.0, 1.0, 0.0),
        60.0
    );

    rl.set_camera_mode(camera, raylib::consts::CameraMode::CAMERA_THIRD_PERSON);

    // ??????????????
    let cam_x = vec3(0.0, 1.0, 0.0).normalized();
    let cam_y = vec3(1.0, 0.0, 0.0).normalized();
    let cam_speed = 0.1;

    // Main game loop
    while !rl.window_should_close() {
        // Input handling
        let mouse_pos = rl.get_mouse_position();

        use raylib::consts::KeyboardKey::*;
        if rl.is_key_down(KEY_E) {
            camera.position += cam_y.scale_by(cam_speed);
            camera.target   += cam_y.scale_by(cam_speed);
        }
        if rl.is_key_down(KEY_S) {
            camera.position -= cam_x.scale_by(cam_speed);
            camera.target   -= cam_x.scale_by(cam_speed);
        }
        if rl.is_key_down(KEY_D) {
            camera.position -= cam_y.scale_by(cam_speed);
            camera.target   -= cam_y.scale_by(cam_speed);
        }
        if rl.is_key_down(KEY_F) {
            camera.position += cam_x.scale_by(cam_speed);
            camera.target   += cam_x.scale_by(cam_speed);
        }

        sq_width += rl.get_mouse_wheel_move() as f32;

        last_mouse_pos = mouse_pos;

        let fill = 1.0;

        // Graphics
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);

        d.draw_text("deeper", 12, 12, 30, Color::WHITE);

        // 3D graphics
        let mut d2 = d.begin_mode_3D(camera);

        for x in 0..=dungeon.width {
            for y in 0..=dungeon.height {
                match dungeon.world.get(&(x, y)) {
                    None => (),
                    Some(value) => match value {
                        &dung_gen::FLOOR => {
                            d2.draw_plane(
                                vec3(x as f32 * sq_width, 0.0, y as f32 * sq_width),
                                vec2(fill * sq_width, fill * sq_width),
                                floor_color,
                            );
                        },
                        &dung_gen::WALL => {
                            d2.draw_cube(
                                vec3(x as f32 * sq_width, sq_width / 2.0, y as f32 * sq_width),
                                fill * sq_width, fill * sq_width, fill * sq_width,
                                wall_color,
                            )
                        }
                        _ => (),
                    }
                }
            }
        }

    }
}
