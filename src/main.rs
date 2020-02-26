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

    let mut sq_width = 20;
    let height_ratio : f32 = 0.43;

    let mut pos_x = vec2(sq_width as f32, 0.0) + vec2(0.0, sq_width as f32 * height_ratio);
    let mut pos_y = vec2(sq_width as f32, 0.0) + vec2(0.0, -sq_width as f32 * height_ratio);

    let mut offset = pos_x.scale_by((-dungeon.width / 4) as f32) + pos_y.scale_by((-dungeon.height / 4) as f32);

    let mut last_mouse_pos = vec2(0.0, 0.0);

    let floor_color = Color::new(50,50,50,255);
    let wall_color_left = Color::new(90,90,90,255);
    let wall_color_right = Color::new(70,70,70,255);
    let wall_color_top = Color::new(128,128,128,255);

    // Main game loop
    while !rl.window_should_close() {
        // Input handling
        use raylib::consts::KeyboardKey::*;
        if rl.is_key_down(KEY_E) {
            offset.y += 1.2;
        }
        if rl.is_key_down(KEY_S) {
            offset.x += 1.2;
        }
        if rl.is_key_down(KEY_D) {
            offset.y -= 1.2;
        }
        if rl.is_key_down(KEY_F) {
            offset.x -= 1.2;
        }
        let mouse_pos = rl.get_mouse_position();
        if rl.is_mouse_button_down(MouseButton::MOUSE_LEFT_BUTTON) {
            offset += mouse_pos - last_mouse_pos;
        }
        sq_width += rl.get_mouse_wheel_move();
        //offset += (pos_x * sq_width as f32 + pos_y * sq_width as f32) * rl.get_mouse_wheel_move() as f32;
        //offset.x -= 5.0 * (sq_width * rl.get_mouse_wheel_move()) as f32;
        //offset.y -= 5.0 * (sq_width * rl.get_mouse_wheel_move()) as f32;

        last_mouse_pos = mouse_pos;

        pos_x = vec2(sq_width as f32, 0.0) + vec2(0.0, sq_width as f32 * height_ratio);
        pos_y = vec2(sq_width as f32, 0.0) + vec2(0.0, -sq_width as f32 * height_ratio);

        // What percentage of the space do the tiles fill
        // Has to get time before d borrows rl
        let fill : f32 = rl.get_time().sin() as f32;

        // Graphics
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::BLACK);
        d.draw_text("deeper", 12, 12, 30, Color::WHITE);

        for x in 0..=dungeon.width {
            for y in 0..=dungeon.height {
                match dungeon.world.get(&(x, y)) {
                    None => (),
                    Some(value) => match value {
                        &dung_gen::FLOOR => {
                            let center = offset.add(pos_x.scale_by(x as f32)).add(pos_y.scale_by(y as f32));
                            let points = [
                                center + vec2(0.0, fill * height_ratio * sq_width as f32),
                                center + vec2(fill * sq_width as f32, 0.0),
                                center + vec2(0.0, -fill * height_ratio * sq_width as f32),
                                center + vec2(-fill * sq_width as f32, 0.0),
                            ];
                            d.draw_triangle_fan(&points, floor_color);
                        }
                        _ => (),
                    }
                }
            }
        }

        for x in 0..=dungeon.width {
            for y in (0..=dungeon.height).rev() {
                match dungeon.world.get(&(x, y)) {
                    None => (),
                    Some(value) => match value {
                        &dung_gen::WALL => {
                            let center = offset.add(pos_x.scale_by(x as f32)).add(pos_y.scale_by(y as f32));
                            let points = [
                                center + vec2(0.0, fill * height_ratio * sq_width as f32 -  2.0 * sq_width as f32 * height_ratio),
                                center + vec2(fill * sq_width as f32, -2.0 * sq_width as f32 * height_ratio),
                                center + vec2(0.0, -fill * height_ratio * sq_width as f32 - 2.0 * sq_width as f32 * height_ratio),
                                center + vec2(-fill * sq_width as f32, -2.0 * sq_width as f32 * height_ratio),
                            ];
                            d.draw_triangle_fan(&points, wall_color_top);
                            let points = [
                                center + vec2(0.0, fill * height_ratio * sq_width as f32 - 2.0 * sq_width as f32 * height_ratio),
                                center + vec2(fill * -sq_width as f32, -2.0 * sq_width as f32 * height_ratio),
                                center + vec2(fill * -sq_width as f32, 0.0),
                                center + vec2(0.0, fill * height_ratio * sq_width as f32),
                            ];
                            d.draw_triangle_fan(&points, wall_color_left);
                            let points = [
                                center + vec2(fill * sq_width as f32, -2.0 * sq_width as f32 * height_ratio),
                                center + vec2(0.0, fill * height_ratio * sq_width as f32 - 2.0 * sq_width as f32 * height_ratio),
                                center + vec2(0.0, fill * height_ratio * sq_width as f32),
                                center + vec2(fill * sq_width as f32, 0.0),
                            ];
                            d.draw_triangle_fan(&points, wall_color_right);
                        },
                        _ => (),
                    }
                }
            }
        }
    }
}
