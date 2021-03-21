#![allow(deprecated)]

use std::time::Instant;

use assman::data::AssetStorageInfo;
use assman::{AssetStore, GraphicsAssetManager};
use cgmath::{InnerSpace, Vector2, Vector3, Zero};
use components::{ActiveCamera, FloorNumber, MapTransition, Player, PlayerCamera, Target};
use entity_smith::{FrameTime, Smith};
use graphics::components::{Camera, Model3D, TemporaryModel3DEntitySmith};
use graphics::models::{ModelQueue, ModelRenderPass};
use input::{CommandManager, InputState};
use physics::PhysicsEntitySmith;
use transforms::{Parent, Scale, SphericalOffset, TransformEntitySmith};
use winit::dpi::PhysicalSize;
use winit::event::{Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use world_gen::components::DynamicModelRequest;

mod misc;

async fn run_async() {
    // Asset Management Initialization

    println!("Starting Asset Manager..");

    let mut ass_man = AssetStore::init();
    let display_settings = ass_man.load_display_settings();

    ass_man.register_assets(None);

    println!("Done registering assets..");

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

    println!("Starting Graphics Layer...");
    // Graphics Initialization
    let mut context = graphics::GraphicsContext::new(&window).await;

    let gui_context = graphics::gui::GuiContext::new(&window, &context);

    let mut graphics_resources = graphics::GraphicsResources::new();

    GraphicsAssetManager::new(&mut ass_man, &mut graphics_resources, &mut context)
        .load_assets_recursive(None);

    println!("Loaded Graphics Assets...");

    let color_texture_id = ass_man
        .get_asset_storage_info("gradient_texture_extended.png")
        .map(|f| match f {
            AssetStorageInfo::Texture(storage_info) => storage_info.unwrap(),
            _ => panic!(),
        })
        .unwrap()
        .id;

    let model_render_pass = ModelRenderPass::new(&context, &graphics_resources, color_texture_id);

    println!("Finished Loading Graphics Layer...");

    // ECS Initialization

    let mut ecs = application::Application::new();

    ecs.create_schedules();

    let mut command_buffer = legion::systems::CommandBuffer::new(&ecs.world);

    let player = command_buffer
        .smith()
        .name("Player")
        .position(Vector3::unit_x())
        .orientation(0.0)
        .agent(5., 30.)
        .velocity(Vector2::zero())
        .dynamic_body(1.)
        .circle_collider(0.3)
        .get_entity();

    let player_model = command_buffer
        .smith()
        .name("Player model")
        .any(Parent(player))
        .orientation(1.0)
        .any(DynamicModelRequest::new("arissa.obj"))
        .any(Scale(0.5))
        .get_entity();

    for &dir in &[
        Vector3::new(1., 1., 0.),
        Vector3::new(-1., 1., 0.),
        Vector3::new(1., -1., 0.),
        Vector3::new(-1., -1., 0.),
    ] {
        command_buffer
            .smith()
            .position(dir.normalize())
            .any(DynamicModelRequest::new("arissa.obj"))
            .any(Scale(0.2))
            .child_of(player_model);
    }

    let player_camera = command_buffer
        .smith()
        .name("The camera")
        .any(Parent(player))
        .any(Target(player))
        .position(Vector3::zero())
        .velocity(Vector2::zero())
        .any(Camera {
            up: Vector3::unit_z(),
            fov: 30.0,
            roaming: false,
        })
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
    ecs.resources.insert(graphics_resources);
    ecs.resources.insert(gui_context);
    ecs.resources.insert(window);
    ecs.resources.insert(ass_man);
    ecs.resources.insert(Instant::now());
    ecs.resources.insert(FrameTime(f32::EPSILON));
    ecs.resources.insert(MapTransition::Deeper);
    ecs.resources.insert(FloorNumber(1));
    ecs.resources.insert(InputState::new());
    ecs.resources.insert(CommandManager::default_bindings());
    ecs.resources.insert(ModelQueue::new());
    ecs.resources.insert(model_render_pass);

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
                    .get_mut::<graphics::GraphicsContext>()
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
