#![allow(deprecated)]

use std::time::Instant;

use application::UnitStage;
use assman::data::AssetStorageInfo;
use assman::systems::AssetManagerBuilderExtender;
use assman::{AssetStore, GraphicsAssetManager};
use cgmath::{InnerSpace, Vector2, Vector3, Zero};
use components::{FloorNumber, MapTransition, Player, PlayerCamera};
use entity_smith::{FrameTime, Smith};
use graphics::canvas::{CanvasQueue, CanvasRenderPipeline};
use graphics::components::{ActiveCamera, Camera, Target};
use graphics::debug::DebugTimer;
use graphics::gui::GuiRenderPipeline;
use graphics::models::{ModelQueue, ModelRenderPipeline};
use graphics::systems::RenderBuilderExtender;
use input::InputState;
use physics::{PhysicsBuilderExtender, PhysicsEntitySmith};
use transforms::{Parent, Scale, SphericalOffset, TransformBuilderExtender, TransformEntitySmith};
use winit::dpi::PhysicalSize;
use winit::event::{Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use world_gen::components::DynamicModelRequest;

async fn run_async() {
    // Asset Management Initialization
    let mut ass_man = AssetStore::init();
    let display_settings = ass_man.load_display_settings();

    ass_man.register_assets(None);

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
    let mut graphics_context = graphics::GraphicsContext::new(&window).await;

    let gui_context = graphics::gui::GuiRenderPipeline::new(&window, &graphics_context);

    let mut graphics_resources = graphics::GraphicsResources::new();

    GraphicsAssetManager::new(&mut ass_man, &mut graphics_resources, &mut graphics_context)
        .load_assets_recursive(None);

    let color_texture_id = ass_man
        .get_asset_storage_info("gradient_texture_extended.png")
        .map(|f| match f {
            AssetStorageInfo::Texture(storage_info) => storage_info.unwrap(),
            _ => panic!(),
        })
        .unwrap()
        .id;

    let model_render_pipeline =
        ModelRenderPipeline::new(&graphics_context, &graphics_resources, color_texture_id);

    let canvas_render_pipeline = CanvasRenderPipeline::new(&graphics_context, &graphics_resources);

    // ECS Initialization
    let mut ecs = {
        let mut builder = application::Application::builder();

        builder.schedule_builders[UnitStage::StartFrame].add_assman_systems();

        builder.schedule_builders[UnitStage::Logic]
            .add_system(systems::player::player_system())
            .add_system(systems::player::camera_control_system())
            .add_system(world_gen::systems::dung_gen_system())
            .add_system(systems::go_to_destination_system())
            .add_physics_systems(&mut builder.world, &mut builder.resources)
            .add_transform_systems();

        builder.schedule_builders[UnitStage::Render].add_render_systems();

        builder
    }
    .with_unit(misc::SnakeUnit)
    .with_unit(input::InputUnit)
    .build();

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
        .any(Target { entity: player })
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
    ecs.resources.insert(graphics_context);
    ecs.resources.insert(graphics_resources);
    ecs.resources.insert(gui_context);
    ecs.resources.insert(window);
    ecs.resources.insert(ass_man);
    ecs.resources.insert(Instant::now());
    ecs.resources.insert(MapTransition::Deeper);
    ecs.resources.insert(FloorNumber(1));
    ecs.resources.insert(ModelQueue::new());
    ecs.resources.insert(CanvasQueue::new());
    ecs.resources.insert(canvas_render_pipeline);
    ecs.resources.insert(model_render_pipeline);

    ecs.resources.insert(0 as i64);

    event_loop.run(move |event, _, control_flow| {
        let imgui_wants_input = {
            let mut gui_context = ecs
                .resources
                .get_mut::<graphics::gui::GuiRenderPipeline>()
                .unwrap();

            gui_context.handle_event(
                &mut *ecs.resources.get_mut::<winit::window::Window>().unwrap(),
                &event,
            );

            gui_context.wants_input()
        };

        match event {
            Event::MainEventsCleared => {
                let frame_time = ecs.resources.get::<Instant>().unwrap().elapsed();

                ecs.resources.insert(FrameTime(frame_time.as_secs_f32()));
                ecs.resources.insert(Instant::now());

                let mut debug_timer = DebugTimer::new();

                debug_timer.push("Frame");

                ecs.resources.insert(debug_timer);

                ecs.resources
                    .get_mut::<GuiRenderPipeline>()
                    .unwrap()
                    .prep_frame(&ecs.resources.get::<winit::window::Window>().unwrap());

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
