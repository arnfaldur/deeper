#![allow(deprecated)]
#![feature(slice_group_by)]

extern crate shaderc;

use std::time::Instant;

use cgmath::{Vector2, Vector3, Zero};
use winit::dpi::PhysicalSize;
use winit::event::{Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

use crate::components::entity_builder::EntitySmith;
use crate::components::*;
use crate::input::{CommandManager, InputState};
use crate::loader::AssetManager;
use crate::transform::components::Position3D;

mod components;
mod debug;
mod dung_gen;
mod ecs;
mod graphics;
mod input;
mod loader;
mod misc;
mod systems;
mod transform;

async fn run_async() {
    // Asset Management Initialization
    let mut ass_man = AssetManager::new();
    let display_settings = ass_man.load_display_settings();

    // Window and Event Creation
    let event_loop = EventLoop::new();

    let size = PhysicalSize {
        width: display_settings.screen_width as u32,
        height: display_settings.screen_height as u32,
    };

    let builder = winit::window::WindowBuilder::new()
        .with_title("deeper")
        .with_inner_size(size);
    let window = builder.build(&event_loop).unwrap();

    // Graphics Initialization
    let mut context = graphics::Context::new(&window).await;

    let gui_context = graphics::gui::GuiContext::new(&window, &context);

    ass_man.load_models(&mut context);

    // ECS Initialization

    let mut ecs = ecs::ECS::new();

    ecs.create_schedules();

    let mut command_buffer = legion::systems::CommandBuffer::new(&ecs.world);

    let player = EntitySmith::from(&mut command_buffer)
        .name("Player")
        .position(Vector3::unit_x())
        .orientation(0.0)
        .agent(5., 30.)
        .velocity(Vector2::zero())
        .dynamic_body(1.)
        .circle_collider(0.3)
        .get_entity();

    let player_model = EntitySmith::from(&mut command_buffer)
        .name("Player model")
        .any(Parent(player))
        .orientation(0.0)
        .model(Model3D::from_index(ass_man.get_model_index("arissa.obj").unwrap()).with_scale(0.5))
        .get_entity();

    let player_camera = EntitySmith::from(&mut command_buffer)
        .name("The camera")
        .any(Parent(player))
        .any(Target(player))
        .position(Vector3::zero())
        .velocity(Vector2::zero())
        .any(components::Camera {
            up: Vector3::unit_z(),
            fov: 30.0,
            roaming: false,
        })
        .any(Position3D(Vector3::new(0.0, 0.0, 0.0)))
        .any(SphericalOffset::camera_offset())
        .get_entity();

    command_buffer.flush(&mut ecs.world, &mut ecs.resources);

    ecs.resources.insert(Player {
        player,
        model: player_model,
    });
    ecs.resources.insert(ActiveCamera {
        entity: player_camera,
    });
    ecs.resources.insert(PlayerCamera {
        entity: player_camera,
    });
    ecs.resources.insert(context);
    ecs.resources.insert(gui_context);
    ecs.resources.insert(window);
    ecs.resources.insert(ass_man);
    ecs.resources.insert(Instant::now());
    ecs.resources.insert(FrameTime(f32::EPSILON));
    ecs.resources.insert(MapTransition::Deeper);
    ecs.resources.insert(FloorNumber(7));
    ecs.resources.insert(InputState::new());
    ecs.resources.insert(CommandManager::default_bindings());

    ecs.resources.insert(0 as i64);

    event_loop.run(move |event, _, control_flow| {
        let imgui_wants_input = {
            let mut gui_context = ecs
                .resources
                .get_mut::<graphics::gui::GuiContext>()
                .unwrap();

            gui_context.handle_event(
                &mut *ecs.resources.get_mut::<winit::window::Window>().unwrap(),
                &event,
            );

            gui_context.wants_input()
        };

        match event {
            Event::MainEventsCleared => {
                ecs.execute_schedules();
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                ecs.resources
                    .get_mut::<graphics::Context>()
                    .unwrap()
                    .resize(size);
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
                if !imgui_wants_input {
                    ecs.resources
                        .get_mut::<InputState>()
                        .unwrap()
                        .update_from_event(&event);
                }
            }
            _ => {
                *control_flow = ControlFlow::Poll;
            }
        }
    });
}

fn main() { futures::executor::block_on(run_async()); }
