use std::fmt::{Display, Formatter};
use std::time::SystemTime;

use legion::systems::Runnable;
use legion::world::SubWorld;
use legion::*;

use crate::components::*;
use crate::graphics;
use crate::graphics::canvas::{AnchorPoint, RectangleDescriptor, ScreenVector};
use crate::loader::AssetManager;
use crate::transform::components::{Position, Position3D};
use crate::transform::Transform;

pub trait RenderBuilderExtender {
    fn add_render_systems(&mut self) -> &mut Self;
}

const DISPLAY_DEBUG_DEFAULT: bool = true;

pub fn render_system_schedule() -> legion::systems::Schedule {
    legion::systems::Schedule::builder()
        .add_thread_local(update_camera_system())
        .add_thread_local(render_draw_static_models_system())
        .add_thread_local(render_draw_models_system())
        .add_thread_local(SnakeSystem::new())
        .add_thread_local(render_system(DISPLAY_DEBUG_DEFAULT))
        .build()
}

fn update_camera_system() -> impl Runnable {
    SystemBuilder::new("update_camera")
        .read_component::<Camera>()
        .read_component::<Position3D>()
        .read_component::<Position>()
        .read_resource::<ActiveCamera>()
        .write_resource::<graphics::Context>()
        .build(move |_, world, resources, _| {
            update_camera(world, &mut *resources.1, &*resources.0);
        })
}

fn update_camera(world: &SubWorld, context: &mut graphics::Context, active_cam: &ActiveCamera) {
    let (cam, cam_pos, cam_target) = {
        <(&Camera, &Position3D, &Position)>::query()
            .get(world, active_cam.entity)
            .unwrap()
    };

    context.set_3d_camera(cam, cam_pos.0, cam_target.0.extend(0.0));
}

#[derive(Clone, Copy)]
enum BoardState {
    Empty,
    Snake,
    Food,
    Wall,
}

impl Display for BoardState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ",
            match self {
                BoardState::Empty => ' ',
                BoardState::Snake => '*',
                BoardState::Food => '%',
                BoardState::Wall => '#',
            }
        )
    }
}

#[derive(Copy, Clone)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

struct SnakeBoard {
    new_dir: Direction,
    player_dir: Direction,
    snake: std::collections::VecDeque<(usize, usize)>,
    board: [[BoardState; 16]; 16],
    food: Option<(usize, usize)>,
}

impl SnakeBoard {
    fn new() -> Self {
        let snake = vec![(4, 5), (4, 4)];

        let mut ret = SnakeBoard {
            new_dir: Direction::Right,
            player_dir: Direction::Right,
            snake: snake.into(),
            board: [[BoardState::Empty; 16]; 16],
            food: None,
        };

        ret.insert_food();
        ret.update_board();

        ret
    }

    fn insert_food(&mut self) {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let mut success = false;
        while !success {
            let test_food = (rng.gen_range(1..15), rng.gen_range(1..15));

            if !self.snake.contains(&test_food) {
                success = true;
                self.food = Some(test_food);
            }
        }
    }

    fn update_board(&mut self) {
        self.board = Self::clear_board();

        for pos in &self.snake {
            self.board[pos.0][pos.1] = BoardState::Snake;
        }

        if let Some(food) = self.food {
            self.board[food.0][food.1] = BoardState::Food;
        }
    }

    fn clear_board() -> [[BoardState; 16]; 16] {
        let mut board = [[BoardState::Empty; 16]; 16];

        for (i, row) in board.iter_mut().enumerate() {
            for (j, square) in row.iter_mut().enumerate() {
                if i % 15 == 0 || j % 15 == 0 {
                    *square = BoardState::Wall;
                }
            }
        }

        board
    }

    fn advance(&mut self) {
        let mut current_pos = self.snake[0];

        self.player_dir = match self.new_dir {
            Direction::Up => match self.player_dir {
                Direction::Down => self.player_dir.clone(),
                _ => self.new_dir,
            },
            Direction::Down => match self.player_dir {
                Direction::Up => self.player_dir.clone(),
                _ => self.new_dir,
            },
            Direction::Left => match self.player_dir {
                Direction::Right => self.player_dir.clone(),
                _ => self.new_dir,
            },
            Direction::Right => match self.player_dir {
                Direction::Left => self.player_dir.clone(),
                _ => self.new_dir,
            },
        };

        match self.player_dir {
            Direction::Up => current_pos.0 -= 1,
            Direction::Down => current_pos.0 += 1,
            Direction::Left => current_pos.1 -= 1,
            Direction::Right => current_pos.1 += 1,
        }

        self.snake.push_front(current_pos);
        self.snake.pop_back();

        match self.board[current_pos.0][current_pos.1] {
            BoardState::Wall | BoardState::Snake => {
                *self = Self::new();
                println!("Oops.");
                return;
            }
            BoardState::Food => {
                self.snake.push_back(*self.snake.back().unwrap());
                self.insert_food();
            }
            _ => (),
        }

        self.update_board();
    }

    fn _print(&self) {
        for row in self.board.iter() {
            for square in row.iter() {
                print!("{}", *square);
            }
            println!();
        }
    }
}

struct SnakeSystem;

impl SnakeSystem {
    fn new() -> impl Runnable { Self::system(SnakeBoard::new(), SystemTime::now()) }

    fn system(mut board: SnakeBoard, mut time: SystemTime) -> impl Runnable {
        SystemBuilder::new("snake_game")
            .read_resource::<crate::input::CommandManager>()
            .write_resource::<crate::graphics::Context>()
            .build(move |_commands, _world, (input, context), _| {
                snake_game(&mut board, &mut time, input, context);

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

fn render_system(mut state_0: bool) -> impl Runnable {
    SystemBuilder::new("render_system")
        .read_resource::<AssetManager>()
        .read_resource::<winit::window::Window>()
        .read_resource::<crate::input::CommandManager>()
        .write_resource::<graphics::gui::GuiContext>()
        .write_resource::<graphics::Context>()
        .write_resource::<crate::debug::DebugTimer>()
        .build(move |_, _, resources, _| {
            render(
                &mut *resources.3,
                &mut *resources.4,
                &*resources.0,
                &*resources.1,
                &mut *resources.5,
                &*resources.2,
                &mut state_0,
            );
        })
}

fn render(
    gui_context: &mut graphics::gui::GuiContext,
    context: &mut graphics::Context,
    ass_man: &AssetManager,
    window: &winit::window::Window,
    debug_timer: &mut crate::debug::DebugTimer,
    input_state: &crate::input::CommandManager,
    toggle: &mut bool,
) {
    use crate::input::Command;
    if input_state.get(Command::DebugToggleInfo) {
        *toggle = !*toggle;
    }

    context.render(ass_man, gui_context, window, debug_timer, *toggle);
}
