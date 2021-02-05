// warnings are really only relevant when doing cleanup
// and are distracting otherwise
// TODO: remove actually fix the warnings
#![allow(warnings)]
// in development code can have some unused variables
// should be periodically removed to remove serious redundancies
#![allow(unused_variables)]
#![allow(unused_must_use)]

extern crate shaderc;

mod components;
mod dung_gen;
mod graphics;
mod input;
mod loader;
mod systems;

use std::time::{Instant, SystemTime};

use cgmath::{Deg, Vector2, Vector3};
use components::*;
use input::InputState;
use legion::{Resources, Schedule, World};
use loader::AssetManager;
use systems::physics::PhysicsBuilderExtender;
use wgpu::SwapChainFrame;
use winit::dpi::PhysicalSize;
use winit::event::{Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;

use crate::graphics::{sc_desc_from_size, GuiContext};
use crate::systems::rendering::RenderBuilderExtender;

async fn run_async() {
    let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);

    let mut ass_man = AssetManager::new();
    let ds = ass_man.load_display_settings();

    let event_loop = EventLoop::new();

    let size = PhysicalSize {
        width: ds.screen_width as u32,
        height: ds.screen_height as u32,
    };

    let builder = winit::window::WindowBuilder::new()
        .with_title("deeper")
        .with_inner_size(size);
    let window = builder.build(&event_loop).unwrap();

    let context = graphics::Context::new(&window, &instance).await;

    let gui_context = graphics::GuiContext::new(&window, &context);

    let surface = unsafe { instance.create_surface(&window) };
    let mut sc_desc = sc_desc_from_size(&size);
    let mut swap_chain = context.device.create_swap_chain(&surface, &sc_desc);

    ass_man.load_models(&context);

    let mut world = World::default();
    let mut resources = Resources::default();

    let mut schedule = Schedule::builder()
        .add_system(systems::assets::hot_loading_system(
            SystemTime::now(),
            false,
        ))
        .add_system(systems::player::player_system())
        .add_system(systems::player::camera_control_system())
        .add_system(systems::go_to_destination_system())
        .add_physics_systems(&mut world, &mut resources)
        .add_system(systems::spherical_offset_system())
        .add_system(systems::world_gen::dung_gen_system())
        .add_render_systems()
        .build();

    let player = world.push((
        Position(Vector2::unit_x()),
        Speed(5.),
        Acceleration(30.),
        Orientation(Deg(0.0)),
        Velocity::new(),
        DynamicBody { mass: 1.0 },
        CircleCollider { radius: 0.3 },
    ));

    if let Some(mut p) = world.entry(player) {
        p.add_component(
            Model3D::from_index(&context, ass_man.get_model_index("arissa.obj").unwrap())
                .with_scale(0.5),
        )
    }

    let player_camera = world.push((
        Parent(player),
        Position(Vector2::unit_x()),
        Speed(5.),
        Acceleration(30.0),
        Velocity::new(),
        components::Camera {
            up: Vector3::unit_z(),
            fov: 30.0,
            roaming: false,
        },
        Position3D(Vector3::new(0.0, 0.0, 0.0)),
        SphericalOffset::camera_offset(),
    ));

    resources.insert(Player { entity: player });
    resources.insert(ActiveCamera {
        entity: player_camera,
    });
    resources.insert(PlayerCamera {
        entity: player_camera,
    });
    resources.insert(context);
    resources.insert(gui_context);
    resources.insert(window);
    resources.insert(ass_man);
    resources.insert(Instant::now());
    resources.insert(SystemTime::now());
    resources.insert(FrameTime(std::f32::EPSILON));
    resources.insert(MapTransition::Deeper);
    resources.insert(FloorNumber(7));
    resources.insert(InputState::new());
    resources.insert(systems::rendering::RenderState::new());

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::MainEventsCleared => {
                // update frametime information
                let frame_time = resources.get::<Instant>().unwrap().elapsed();
                resources.insert(FrameTime(frame_time.as_secs_f32()));
                resources.insert(Instant::now());
                resources.insert(sc_desc.clone());
                // Explicitly drop the current swap frame in preparation
                // for the next.
                drop(resources.remove::<wgpu::SwapChainFrame>());
                resources.insert(swap_chain.get_current_frame().unwrap());
                schedule.execute(&mut world, &mut resources);
                resources.get_mut::<InputState>().unwrap().new_frame();
                schedule.execute(&mut world, &mut resources);
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                sc_desc = sc_desc_from_size(&size);
                swap_chain = resources
                    .get_mut::<graphics::Context>()
                    .unwrap()
                    .resize(size, &sc_desc, &surface);
            }
            // note(Jökull): Can we make this more readable somehow?
            // It is not clear that these two events result in Exit
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            }
            | Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    },
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::WindowEvent { ref event, .. } => {
                resources
                    .get_mut::<InputState>()
                    .unwrap()
                    .update_from_event(&event);
            }
            _ => {
                *control_flow = ControlFlow::Poll;
            }
        }

        resources.get_mut::<GuiContext>().unwrap().handle_event(
            &mut *resources.get_mut::<winit::window::Window>().unwrap(),
            &event,
        )
    });
}

fn main() { futures::executor::block_on(run_async()); }
