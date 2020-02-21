
mod dung_gen;

use dung_gen::DungGen;
use dung_gen::TileKind;

use raylib::prelude::*;


fn main() {
    let dungeon = DungGen::new()
        .width(150)
        .height(150)
        .n_rooms(120)
        .room_min(10)
        .room_range(15)
        .generate();

    // dungeon.print();

    let (mut rl, thread) = raylib::init()
        .size(1024, 768)
        .title("deeper")
        .build();

    let sq_width = 10;

    let mut x_offset = 0.25 * -sq_width as f64 * dungeon.width  as f64;
    let mut y_offset = 0.25 * -sq_width as f64 * dungeon.height as f64;

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::BLACK);
        d.draw_text("deeper", 12, 12, 30, Color::WHITE);

        x_offset += 0.3;
        y_offset += 0.2;

        if x_offset >= (sq_width * dungeon.width)  as f64 { x_offset = -0.75 * (sq_width * dungeon.width)  as f64 }
        if y_offset >= (sq_width * dungeon.height) as f64 { y_offset = -0.75 * (sq_width * dungeon.height) as f64 }

        for x in 0..=dungeon.height {
            for y in 0..=dungeon.width {
                match dungeon.world.get(&(x, y)) {
                    None => (),
                    Some(value) => match value {
                        &dung_gen::WALL => d.draw_rectangle(x_offset as i32 + x * sq_width, y_offset as i32 + y * sq_width, sq_width, sq_width, Color::WHITE),
                        _ => (),
                    }
                }
            }
        }
    }
}
