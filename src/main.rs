mod loader;
use loader::AssetManager;

mod dung_gen;
use dung_gen::DungGen;

mod graphics;
mod input;

mod components;
mod systems;

use crate::components::components::*;
use crate::systems::systems::*;

use std::f32::consts::PI;
use rand::seq::SliceRandom;
use specs::prelude::*;

use winit::event_loop::{EventLoop, ControlFlow};
use winit::dpi::{PhysicalSize};
use winit::event::{Event, WindowEvent, KeyboardInput, VirtualKeyCode};

use std::{mem, slice};
use crate::graphics::{Vertex, Model, Mesh};
use wgpu::{TextureViewDimension, CompareFunction, PrimitiveTopology, BufferDescriptor, CommandEncoder};
use cgmath::{Vector2, Vector3};

use zerocopy::AsBytes;
use crate::input::{EventBucket, InputState};
use rand::{thread_rng, Rng};


async fn run_async() {
    let mut ass_man = AssetManager::new();
    let ds = ass_man.load_display_settings();

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

    let event_loop = EventLoop::new();

    let size = PhysicalSize { width: ds.screen_width, height: ds.screen_height };

    let mut builder = winit::window::WindowBuilder::new()
        .with_title("deeper")
        .with_inner_size(size);
    let window = builder.build(&event_loop).unwrap();

    let context = graphics::Context::new(&window).await;

    ass_man.load_models(&context);

    let mut init_encoder = context.device.create_command_encoder(
        &wgpu::CommandEncoderDescriptor { todo: 0 }
    );

    let mut lights: graphics::Lights = Default::default();

    lights.directional_light = graphics::DirectionalLight {
        direction: [1.0, 0.8, 0.8, 0.0],
        ambient: [0.1, 0.1, 0.1, 1.0],
        color: [0.2, 0.2, 0.3, 1.0],
    };

    for (i, &(x, y)) in dungeon.room_centers.iter().enumerate() {
        if i >= graphics::MAX_NR_OF_POINT_LIGHTS { break; }
        lights.point_lights[i] = Default::default();
        lights.point_lights[i].radius = 20.0;
        lights.point_lights[i].position = [x as f32, y as f32, 5.0, 1.0];
        lights.point_lights[i].color = [0.6, 0.4, 0.1, 1.0];
    }

    let temp_buf = context.device.create_buffer_with_data(
        lights.as_bytes(),
        wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_SRC,
    );

    init_encoder.copy_buffer_to_buffer(
        &temp_buf,
        0,
        &context.lights_buf,
        0,
        std::mem::size_of::<graphics::Lights>() as u64,
    );

    let command_buffer = init_encoder.finish();

    context.queue.submit(&[command_buffer]);
    // End graphics shit

    let mut world = World::new();

    register_components(&mut world);

    use std::path::Path;

    // initialize dispacher with all game systems
    let mut dispatcher = DispatcherBuilder::new()
        .with(DunGenSystem { dungeon }, "DunGenSystem", &[])
        .with(HotLoaderSystem::new(), "HotLoaderSystem", &[])
        .with(PlayerSystem::new(), "PlayerSystem", &[])
        .with(AIFollowSystem, "AIFollowSystem", &[])
        .with(GoToDestinationSystem, "GoToDestinationSystem", &["AIFollowSystem"])
        .with(Physics2DSystem, "Physics2DSystem", &["GoToDestinationSystem", "PlayerSystem", "AIFollowSystem"])
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
        .with_thread_local(GraphicsSystem)
        .build();

    let player = world
        .create_entity()
        .with(Position(Vector2::new(player_start.0 as f32, player_start.1 as f32)))
        .with(Speed(0.05))
        .with(Acceleration(0.01))
        .with(Orientation(0.0))
        .with(Velocity::new())
        .with(DynamicBody)
        .with(CircleCollider { radius: 0.3 })
        .with(Model3D::from_index(&context, ass_man.get_model_index("arissa.obj").unwrap()).with_scale(0.5))
        .build();

    let player_camera = world
        .create_entity()
        .with(components::components::Camera {
            up: Vector3::unit_z(),
            fov: 25.0,
        })
        .with(Target(player))
        .with(Position3D(Vector3::new(0.0, 0.0, 0.0)))
        .with(SphericalOffset::new())
        .build();

    let mut rng = thread_rng();
    for _enemy in 0..128 {
        let (randx, randy): (f32, f32) = rng.gen();
        world.create_entity()
            .with(Position(Vector2::new(
                player_start.0 as f32 + (randx) * 4.0,
                player_start.1 as f32 + (randy) * 4.0,
            )))
            .with(Speed(0.02))
            .with(Acceleration(0.0005))
            .with(Orientation(0.0))
            .with(Velocity::new())
            .with(DynamicBody)
            .with(CircleCollider { radius: 0.1 })
            .with(AIFollow {
                target: player,
                minimum_distance: 1.0,
            })
            .with(Model3D::from_index(&context, ass_man.get_model_index("sphere2.obj").unwrap())
                .with_scale(0.1)
                .with_material(graphics::Material::glossy(Vector3::<f32>::new(rng.gen(), rng.gen(), rng.gen()))))
            .build();
    }

    world.insert(Player::from_entity(player));
    world.insert(ActiveCamera(player_camera));
    world.insert(PlayerCamera(player_camera));
    world.insert(context);
    world.insert(ass_man);

    let input_state = InputState::new();
    world.insert(input_state);

    // Setup world
    dispatcher.setup(&mut world);

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::MainEventsCleared => {
                dispatcher.dispatch(&mut world);
                world.get_mut::<InputState>().unwrap().new_frame();
                world.maintain();
            },
            Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
                world.get_mut::<graphics::Context>().unwrap().resize(size);
                //unimplemented!();
            },
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. }
            | Event::WindowEvent  {
                event: WindowEvent::KeyboardInput {
                    input: KeyboardInput {
                        virtual_keycode: Some(VirtualKeyCode::Escape), ..
                    }, ..
                }, ..
            }
                => *control_flow = ControlFlow::Exit,
            Event::WindowEvent { event, .. } => {
                world.get_mut::<InputState>().unwrap().update_from_event(&event);
            }
            _ => {
                *control_flow = ControlFlow::Poll;
            }
        }
    });
}

fn main() {
    futures::executor::block_on(run_async());
}
