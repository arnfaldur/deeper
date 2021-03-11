use crate::data::{DirectionalLight, Lights, PointLight};
use crate::{Context, MAX_NR_OF_POINT_LIGHTS};

pub fn make_lights(context: &&mut Context, room_centers: &Vec<(i32, i32)>) {
    let mut lights: Lights = Default::default();

    lights.directional_light = DirectionalLight {
        direction: [1.0, 0.8, 0.8, 0.0],
        ambient: [0.01, 0.015, 0.02, 1.0],
        color: [0.02, 0.025, 0.05, 1.0],
    };

    for (i, &(x, y)) in room_centers.iter().enumerate() {
        if i >= MAX_NR_OF_POINT_LIGHTS {
            break;
        }
        lights.point_lights[i] = PointLight {
            position: [x as f32, y as f32, 5.0, 1.0],
            radius: 10.0,
            color: [2.0, 1.0, 0.1, 1.0],
            ..Default::default()
        };
    }

    context.queue.write_buffer(
        &context.model_render_ctx.lights_uniform_buf,
        0,
        bytemuck::bytes_of(&lights),
    );
}
