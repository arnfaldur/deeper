// in development code can have some unused variables
// should be periodically removed to remove serious redundancies
#![allow(unused_variables)]
// TODO: remove actually fix the warnings
#![allow(unused_must_use)]

mod components;
mod dung_gen;
mod graphics;
mod input;
mod loader;
mod systems;

use std::ops::DerefMut;
use std::time::Instant;
use std::time::SystemTime;

use legion::{Resources, Schedule, World};

use winit::dpi::PhysicalSize;
use winit::event::{Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

use cgmath::{Deg, Vector2, Vector3};

use nphysics2d::force_generator::DefaultForceGeneratorSet;
use nphysics2d::joint::DefaultJointConstraintSet;
use nphysics2d::object::{DefaultBodySet, DefaultColliderSet};
use nphysics2d::world::{DefaultGeometricalWorld, DefaultMechanicalWorld};

use components::*;
use input::InputState;
use loader::AssetManager;
use systems::spherical_offset;
//use crate::systems::assets::*;

use systems::physics::PhysicsBuilderExtender;

async fn run_async() {
    let mut ass_man = AssetManager::new();
    let ds = ass_man.load_display_settings();

    let event_loop = EventLoop::new();

    let size = PhysicalSize {
        width: ds.screen_width,
        height: ds.screen_height,
    };

    let builder = winit::window::WindowBuilder::new()
        .with_title("deeper")
        .with_inner_size(size);
    let window = builder.build(&event_loop).unwrap();

    let context = graphics::Context::new(&window).await;

    ass_man.load_models(&context);

    let mut world = World::default();
    let mut resources = Resources::default();
    println!("present");
    let mut body: nphysics2d::object::RigidBodyDesc<f32> = nphysics2d::object::RigidBodyDesc::new();
    println!("past");

    // initialize dispatcher with all game systems
    //let mut dispatcher = DispatcherBuilder::new()
    //    .with(assets::HotLoaderSystem::new(), "HotLoader", &[])
    //    .with(player::PlayerSystem, "Player", &[])
    //    .with(player::CameraControlSystem, "CameraControl", &["Player"])
    //    .with(HitPointRegenSystem, "HitPointRegen", &["Player"])
    //    .with(AIFollowSystem, "AIFollow", &[])
    //    .with(GoToDestinationSystem, "GoToDestination", &["AIFollow"])
    //    .with(physics::Physics2DSystem, "Physics2D", &["GoToDestination", "Player", "AIFollow"])
    //    .with(physics::MovementSystem, "Movement", &["Physics2D", "Player"])
    //    .with(SphericalOffsetSystem, "SphericalFollow", &["Movement"])
    //    .with(world_gen::MapSwitchingSystem, "MapSwitching", &["Movement"])
    //    .with(world_gen::DunGenSystem, "DunGen", &["MapSwitching"])
    //    .with_thread_local(rendering::RenderingSystem::new())
    //    .build();

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
        .add_system(systems::rendering::rendering_system(SystemTime::now()))
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
    resources.insert(ass_man);
    resources.insert(Instant::now());
    resources.insert(FrameTime(std::f32::EPSILON));
    resources.insert(MapTransition::Deeper);
    resources.insert(FloorNumber(8));
    resources.insert(InputState::new());

    // Setup world
    //dispatcher.setup(&mut world);

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::MainEventsCleared => {
                // update frametime information
                let frame_time = resources.get::<Instant>().unwrap().elapsed();
                resources.insert(FrameTime(frame_time.as_secs_f32()));
                resources.insert(Instant::now());
                schedule.execute(&mut world, &mut resources);
                resources.get_mut::<InputState>().unwrap().new_frame();
                //world.write_resource::<FrameTime>().0 = frame_time.as_secs_f32();
                //*world.write_resource::<Instant>().deref_mut() = Instant::now();
                //dispatcher.dispatch(&mut world);
                //world.get_mut::<InputState>().unwrap().new_frame();
                //world.maintain();
                schedule.execute(&mut world, &mut resources);
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                resources
                    .get_mut::<graphics::Context>()
                    .unwrap()
                    .resize(size);
                //world.get_mut::<graphics::Context>().unwrap().resize(size);
            }
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
            Event::WindowEvent { event, .. } => {
                resources
                    .get_mut::<InputState>()
                    .unwrap()
                    .update_from_event(&event);
                //world.get_mut::<InputState>().unwrap().update_from_event(&event);
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
