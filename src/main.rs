mod dung_gen;

use dung_gen::{
    DungGen,
    TileKind,
};

use raylib::prelude::*;


fn main() {
    let dungeon = DungGen::new()
        .width(150)
        .height(150)
        .n_rooms(20)
        .room_min(10)
        .room_range(15)
        .generate()
        .print();

    let (mut rl, thread) = raylib::init()
        .size(1024, 768)
        .resizable()
        .title("deeper")
        .build();

    let sq_width = 10;

    let scroll_rate = 0.03;
    let mut offset = vec2(0.25 * -sq_width as f32 * dungeon.width as f32,
                          0.25 * -sq_width as f32 * dungeon.height as f32);
    let mut scale = 1.0;

    let mut last_mouse_pos = vec2(0.0, 0.0);

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
        let delta_zoom = rl.get_mouse_wheel_move() as f32 * scroll_rate;
        offset += (offset - rl.get_mouse_position()) * delta_zoom;
        scale *= 1.0 + delta_zoom;
        if rl.is_mouse_button_pressed(MouseButton::MOUSE_RIGHT_BUTTON) {
            offset = vec2(0.0, 0.0);
            scale = 1.0;
        }
        let mouse_pos = rl.get_mouse_position();
        if rl.is_mouse_button_down(MouseButton::MOUSE_LEFT_BUTTON) {
            offset += mouse_pos - last_mouse_pos;
        }
        last_mouse_pos = mouse_pos;

        // Graphics
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::BLACK);
        d.draw_text("deeper", 12, 12, 30, Color::WHITE);


        if offset.x >= (sq_width * dungeon.width) as f32 { offset.x = -0.75 * (sq_width * dungeon.width) as f32 }
        if offset.y >= (sq_width * dungeon.height) as f32 { offset.y = -0.75 * (sq_width * dungeon.height) as f32 }

        for x in 0..=dungeon.height {
            for y in 0..=dungeon.width {
                match dungeon.world.get(&(x, y)) {
                    None => (),
                    Some(value) => match value {
                        &dung_gen::WALL => d.draw_rectangle(offset.x as i32 + x * sq_width, offset.y as i32 + y * sq_width, sq_width, sq_width, Color::WHITE),
                        _ => (),
                    }
                }
            }
        }
    }
}
