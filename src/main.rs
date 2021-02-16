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

use std::rc::Rc;
use std::time::{Instant, SystemTime};

use cgmath::{Deg, Vector2, Vector3, Zero};
use components::*;
use input::InputState;
use legion::storage::{ComponentSource, EntityLayout, IntoComponentSource};
use legion::{Resources, Schedule, World};
use loader::AssetManager;
use systems::physics::PhysicsBuilderExtender;
use wgpu::SwapChainFrame;
use winit::dpi::PhysicalSize;
use winit::event::{Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;

use crate::components::entity_builder::EntityBuilder;
use crate::systems::rendering::RenderBuilderExtender;

async fn run_async() {
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

    let context = graphics::Context::new(&window).await;

    let gui_context = graphics::gui::GuiContext::new(&window, &context);

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
        .add_system(systems::assets::hot_loading_system(
            SystemTime::now(),
            false,
        ))
        .add_render_systems()
        .build();

    let mut command_buffer = legion::systems::CommandBuffer::new(&world);

    let player = EntityBuilder::from_buffer(&mut command_buffer)
        .position(Vector2::unit_x())
        .orientation(0.0)
        .agent(5., 30.)
        .velocity(Vector2::zero())
        .dynamic_body(1.)
        .circle_collider(0.3)
        .model(
            Model3D::from_index(&context, ass_man.get_model_index("arissa.obj").unwrap())
                .with_scale(0.5),
        )
        .build();

    let player_camera = EntityBuilder::from_buffer(&mut command_buffer)
        .any(Parent(player))
        .position(Vector2::unit_x())
        .velocity(Vector2::zero())
        .any(components::Camera {
            up: Vector3::unit_z(),
            fov: 30.0,
            roaming: false,
        })
        .any(Position3D(Vector3::new(0.0, 0.0, 0.0)))
        .any(SphericalOffset::camera_offset())
        .build();

    command_buffer.flush(&mut world);

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
    resources.insert(FrameTime(f32::EPSILON));
    resources.insert(MapTransition::Deeper);
    resources.insert(FloorNumber(7));
    resources.insert(InputState::new());

    event_loop.run(move |event, _, control_flow| {
        let imgui_wants_input = {
            let mut gui_context = resources.get_mut::<graphics::gui::GuiContext>().unwrap();

            gui_context.handle_event(
                &mut *resources.get_mut::<winit::window::Window>().unwrap(),
                &event,
            );

            gui_context.wants_input()
        };

        match event {
            Event::MainEventsCleared => {
                let frame_time = resources.get::<Instant>().unwrap().elapsed();
                resources.insert(FrameTime(frame_time.as_secs_f32()));
                resources.insert(Instant::now());

                schedule.execute(&mut world, &mut resources);

                resources.get_mut::<InputState>().unwrap().new_frame();
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                resources
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
                    resources
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
