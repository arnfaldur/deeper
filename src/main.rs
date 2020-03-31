mod loader;

use loader::AssetManager;

mod dung_gen;

use dung_gen::DungGen;

mod graphics;
mod input;

mod components;
mod systems;

use crate::components::*;
use crate::systems::systems::*;

use std::f32::consts::PI;
use rand::seq::SliceRandom;
use specs::prelude::*;

use winit::event_loop::{EventLoop, ControlFlow};
use winit::dpi::PhysicalSize;
use winit::event::{Event, WindowEvent, KeyboardInput, VirtualKeyCode};

use std::{mem, slice};
use crate::graphics::{Vertex, Model, Mesh};
use wgpu::{TextureViewDimension, CompareFunction, PrimitiveTopology, BufferDescriptor, CommandEncoder};
use cgmath::{Vector2, Vector3, Deg};

use zerocopy::AsBytes;
use crate::input::{EventBucket, InputState};
use rand::{thread_rng, Rng};
use std::time::Instant;
use glsl_to_spirv::ShaderType::Fragment;
use std::ops::DerefMut;


async fn run_async() {
    let mut ass_man = AssetManager::new();
    let ds = ass_man.load_display_settings();


    let event_loop = EventLoop::new();

    let size = PhysicalSize { width: ds.screen_width, height: ds.screen_height };

    let mut builder = winit::window::WindowBuilder::new()
        .with_title("deeper")
        .with_inner_size(size);
    let window = builder.build(&event_loop).unwrap();

    let context = graphics::Context::new(&window).await;

    ass_man.load_models(&context);

    let mut world = World::new();

    register_components(&mut world);

    use std::path::Path;

    use rg3d_sound::context::Context as AudioContext;

    let ac = AudioContext::new().unwrap();

    // initialize dispacher with all game systems
    let mut dispatcher = DispatcherBuilder::new()
        .with(HotLoaderSystem::new(), "HotLoader", &[])
        .with(PlayerSystem, "Player", &[])
        .with(HitPointRegenSystem, "HitPointRegen", &["Player"])
        .with(AIFollowSystem, "AIFollow", &[])
        .with(GoToDestinationSystem, "GoToDestination", &["AIFollow"])
        .with(Physics2DSystem, "Physics2D", &["GoToDestination", "Player", "AIFollow"])
        .with(MovementSystem, "Movement", &["Physics2D", "Player"], )
        .with(SphericalFollowSystem, "SphericalFollow", &["Movement"], )
        .with(MapSwitchingSystem, "MapSwitching", &["Movement"])
        .with(DunGenSystem, "DunGen", &["MapSwitching"])
        .with(GraphicsSystem, "Graphics", &[]).build();

    let player = world
        .create_entity()
        .with(Position(Vector2::unit_x()))
        .with(Speed(5.))
        .with(Acceleration(30.))
        .with(Orientation(Deg(0.0)))
        .with(Velocity::new())
        .with(DynamicBody(1.0))
        .with(CircleCollider { radius: 0.3 })
        .with(Model3D::from_index(&context, ass_man.get_model_index("arissa.obj").unwrap()).with_scale(0.5))
        .build();

    let player_camera = world
        .create_entity()
        .with(components::Camera {
            up: Vector3::unit_z(),
            fov: 25.0,
        })
        .with(Target(player))
        .with(Position3D(Vector3::new(0.0, 0.0, 0.0)))
        .with(SphericalOffset::new())
        .build();

    world.insert(Player::from_entity(player));
    world.insert(ActiveCamera(player_camera));
    world.insert(PlayerCamera(player_camera));
    world.insert(context);
    world.insert(ass_man);
    world.insert(Instant::now());
    world.insert(FrameTime(std::f32::EPSILON));
    world.insert(MapTransition::Deeper);
    world.insert(20 as i64);

    let input_state = InputState::new();
    world.insert(input_state);

    // Setup world
    dispatcher.setup(&mut world);

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::MainEventsCleared => {
                // update frametime information
                let frame_time = world.read_resource::<Instant>().elapsed();
                world.write_resource::<FrameTime>().0 = frame_time.as_secs_f32();
                *world.write_resource::<Instant>().deref_mut() = Instant::now();
                dispatcher.dispatch(&mut world);
                world.get_mut::<InputState>().unwrap().new_frame();
                world.maintain();
            }
            Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
                world.get_mut::<graphics::Context>().unwrap().resize(size);
                //unimplemented!();
            }
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. }
            | Event::WindowEvent {
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
