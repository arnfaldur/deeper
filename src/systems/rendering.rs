use std::fmt::{Display, Formatter};
use std::time::SystemTime;

use legion::systems::Runnable;
use legion::world::SubWorld;
use legion::*;

use crate::components::*;
use crate::graphics;
use crate::graphics::canvas::{AnchorPoint, RectangleDescriptor, ScreenVector};
use crate::loader::AssetManager;

pub trait RenderBuilderExtender {
    fn add_render_systems(&mut self) -> &mut Self;
}

impl RenderBuilderExtender for legion::systems::Builder {
    fn add_render_systems(&mut self) -> &mut Self {
        self.add_thread_local(update_camera_system())
            .add_thread_local(render_draw_static_models_system())
            .add_thread_local(render_draw_models_system())
            .add_thread_local(render_gui_init_system())
            .add_thread_local(render_gui_test_system())
            .add_thread_local(SnakeSystem::new())
            .add_thread_local(render_system())
    }
}

#[system]
#[read_component(Camera)]
#[read_component(Position3D)]
#[read_component(WorldPosition)]
fn update_camera(
    world: &SubWorld,
    #[resource] context: &mut graphics::Context,
    #[resource] active_cam: &ActiveCamera,
) {
    let (cam, cam_pos, cam_target) = {
        <(&Camera, &Position3D, &WorldPosition)>::query()
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
    fn new() -> impl Runnable { Self::system(SnakeBoard::new(), SystemTime::now(), false) }

    fn system(mut board: SnakeBoard, mut time: SystemTime, mut toggle: bool) -> impl Runnable {
        SystemBuilder::new("snake_game")
            .read_resource::<crate::input::InputState>()
            .write_resource::<crate::graphics::Context>()
            .build(move |_commands, _world, (input_state, context), _| {
                snake_game(&mut board, &mut time, &mut toggle, input_state, context);

                fn snake_game(
                    board: &mut SnakeBoard,
                    time: &mut SystemTime,
                    toggle: &mut bool,
                    input_state: &crate::input::InputState,
                    context: &mut graphics::Context,
                ) {
                    use crate::input::Key;

                    if input_state.is_key_pressed(Key::P) {
                        *toggle = !*toggle;
                    }

                    if !*toggle {
                        return;
                    }

                    let key_up = input_state.is_key_pressed(Key::Up);
                    let key_down = input_state.is_key_pressed(Key::Down);
                    let key_left = input_state.is_key_pressed(Key::Left);
                    let key_right = input_state.is_key_pressed(Key::Right);

                    if key_up {
                        board.new_dir = Direction::Up;
                    } else if key_down {
                        board.new_dir = Direction::Down;
                    } else if key_left {
                        board.new_dir = Direction::Left;
                    } else if key_right {
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

#[system(for_each)]
fn render_draw_models(
    model: &Model3D,
    position: &WorldPosition,
    orientation: Option<&Orientation>,
    #[resource] context: &mut graphics::Context,
) {
    context.draw_model(
        model,
        position.into(),
        orientation.and(Option::from(orientation.unwrap().0)),
    );
}

#[system(for_each)]
fn render_draw_static_models(model: &StaticModel, #[resource] context: &mut graphics::Context) {
    context.draw_static_model(model.clone());
}

#[system]
fn render_gui_init(
    #[resource] gui_context: &mut graphics::gui::GuiContext,
    #[resource] window: &winit::window::Window,
) {
    gui_context.prep_frame(window);
    gui_context.new_frame();
}

#[system]
fn render_gui_test(#[resource] _gui_context: &mut graphics::gui::GuiContext) {
    graphics::gui::GuiContext::with_ui(|ui| {
        use imgui::{im_str, Condition};
        let test_window = imgui::Window::new(im_str!("Test Window"));

        test_window
            .size([300.0, 100.0], Condition::FirstUseEver)
            .build(ui, || {
                ui.text("Welcome to deeper.");
            });
    });
}

#[system]
fn render(
    #[resource] gui_context: &mut graphics::gui::GuiContext,
    #[resource] context: &mut graphics::Context,
    #[resource] ass_man: &AssetManager,
    #[resource] window: &winit::window::Window,
) {
    context.render(ass_man, gui_context, window);
}
