use std::time::SystemTime;

use graphics;
use graphics::assets::AssetManager;
use graphics::canvas::{AnchorPoint, RectangleDescriptor, ScreenVector};
use legion::systems::Runnable;
use legion::*;
use transforms::{Position, Transform};
use graphics::components::{Camera, Model3D, StaticModel};
use components::{Target, ActiveCamera};

use crate::misc;

pub trait RenderBuilderExtender {
    fn add_render_systems(&mut self) -> &mut Self;
}

pub fn render_system_schedule() -> legion::systems::Schedule {
    legion::systems::Schedule::builder()
        .add_thread_local(update_camera_system())
        .add_thread_local(render_draw_static_models_system())
        .add_thread_local(render_draw_models_system())
        .add_thread_local(SnakeSystem::new())
        .add_thread_local(render_system())
        .build()
}

fn update_camera_system() -> impl Runnable {
    SystemBuilder::new("update_camera")
        .read_component::<Camera>()
        .read_component::<Position>()
        .read_component::<Transform>()
        .read_component::<Target>()
        .read_resource::<ActiveCamera>()
        .write_resource::<graphics::Context>()
        .build(move |_, world, (active_cam, context), _| {
            if let Ok((cam, cam_pos, target)) =
                <(&Camera, &Transform, &Target)>::query().get(world, active_cam.entity)
            {
                if let Ok(target_pos) = <&Transform>::query().get(world, target.0) {
                    context.set_3d_camera(
                        cam,
                        cam_pos.absolute.w.truncate(),
                        target_pos.absolute.w.truncate(),
                    );
                }
            }
        })
}

struct SnakeSystem;



impl SnakeSystem {
    fn new() -> impl Runnable { Self::system(misc::SnakeBoard::new(), SystemTime::now()) }

    fn system(mut board: misc::SnakeBoard, mut time: SystemTime) -> impl Runnable {
        SystemBuilder::new("snake_game")
            .read_resource::<crate::input::CommandManager>()
            .write_resource::<graphics::Context>()
            .build(move |_commands, _world, (input, context), _| {
                snake_game(&mut board, &mut time, input, context);

                use misc::*;

                fn snake_game(
                    board: &mut SnakeBoard,
                    time: &mut SystemTime,
                    input: &crate::input::CommandManager,
                    context: &mut graphics::Context,
                ) {
                    use crate::input::Command;

                    if !input.get(Command::DebugToggleSnake) {
                        return;
                    }

                    if input.get(Command::SnakeMoveUp) {
                        board.new_dir = Direction::Up;
                    } else if input.get(Command::SnakeMoveDown) {
                        board.new_dir = Direction::Down;
                    } else if input.get(Command::SnakeMoveLeft) {
                        board.new_dir = Direction::Left;
                    } else if input.get(Command::SnakeMoveRight) {
                        board.new_dir = Direction::Right;
                    }

                    if SystemTime::now().duration_since(*time).unwrap().as_millis() > 200 {
                        *time = SystemTime::now();
                        board.advance();
                    }

                    for (i, row) in board.board.iter().enumerate() {
                        for (j, square) in row.iter().enumerate() {
                            context.canvas_queue.draw_rect(
                                RectangleDescriptor::AnchorRect {
                                    anchor: AnchorPoint::TopLeft,
                                    position: ScreenVector::new_relative(0.5, 0.5),
                                    dimensions: ScreenVector::new_relative_to_width(0.025, 0.025),
                                    offset: ScreenVector::new_relative_to_width(
                                        (j as f32 - 8.0) / 16.0 / 2.0,
                                        (i as f32 - 8.0) / 16.0 / 2.0,
                                    ),
                                },
                                match square {
                                    BoardState::Empty => cgmath::Vector4::new(0.1, 0.1, 0.1, 1.0),
                                    BoardState::Snake => cgmath::Vector4::new(0.8, 0.2, 0.2, 1.0),
                                    BoardState::Food => cgmath::Vector4::new(0.8, 0.8, 0.2, 1.0),
                                    BoardState::Wall => cgmath::Vector4::new(0.2, 0.2, 0.2, 1.0),
                                },
                                context.window_size,
                            );
                        }
                    }
                }
            })
    }
}

fn render_draw_models_system() -> impl Runnable {
    SystemBuilder::new("render_draw_models")
        .write_resource::<graphics::Context>()
        .with_query(<(&Model3D, &Transform)>::query())
        .build(move |_, world, resources, query| {
            let for_query = world;
            query.for_each_mut(for_query, |components| {
                render_draw_models(components.0, components.1, &mut *resources);
            });
        })
}

fn render_draw_models(
    model: &Model3D,
    transform: &Transform,
    // position: &Position,
    // orientation: Option<&Rotation>,
    context: &mut graphics::Context,
) {
    context.draw_model(
        model,
        transform.absolute,
        // position.into(),
        // orientation.and(Option::from(orientation.unwrap().0)),
    );
}

fn render_draw_static_models_system() -> impl Runnable {
    SystemBuilder::new("render_draw_static_models_system")
        .write_resource::<graphics::Context>()
        .with_query(<&StaticModel>::query())
        .build(move |_, world, resources, query| {
            let for_query = world;
            query.for_each_mut(for_query, |components| {
                render_draw_static_models(components, &mut *resources);
            });
        })
}

fn render_draw_static_models(model: &StaticModel, context: &mut graphics::Context) {
    context.draw_static_model(model.clone());
}

fn render_system() -> impl Runnable {
    SystemBuilder::new("render_system")
        .read_resource::<AssetManager>()
        .read_resource::<winit::window::Window>()
        .read_resource::<crate::input::CommandManager>()
        .write_resource::<graphics::gui::GuiContext>()
        .write_resource::<graphics::Context>()
        .write_resource::<graphics::debug::DebugTimer>()
        .build(
            move |_,
                  _,
                  (ass_man, window, command_manager, gui_context, context, debug_timer),
                  _| {
                render(
                    gui_context,
                    context,
                    ass_man,
                    window,
                    debug_timer,
                    command_manager,
                );
            },
        )
}

fn render(
    gui_context: &mut graphics::gui::GuiContext,
    context: &mut graphics::Context,
    ass_man: &AssetManager,
    window: &winit::window::Window,
    debug_timer: &mut graphics::debug::DebugTimer,
    input_state: &crate::input::CommandManager,
) {
    use crate::input::Command;

    context.render(
        ass_man,
        gui_context,
        window,
        debug_timer,
        input_state.get(Command::DebugToggleInfo),
    );
}
