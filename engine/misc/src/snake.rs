#![allow(dead_code)]

use std::time::SystemTime;

use graphics::canvas::{AnchorPoint, CanvasQueue, RectangleDescriptor, ScreenVector};
use graphics::GraphicsContext;
use input::{Command, CommandManager};
use legion::systems::Runnable;
use legion::SystemBuilder;

#[derive(Clone, Copy)]
pub(crate) enum BoardState {
    Empty,
    Snake,
    Food,
    Wall,
}

impl std::fmt::Display for BoardState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
pub(crate) enum Direction {
    Up,
    Down,
    Left,
    Right,
}

pub(crate) struct SnakeBoard {
    pub(crate) new_dir: Direction,
    player_dir: Direction,
    snake: std::collections::VecDeque<(usize, usize)>,
    pub(crate) board: [[BoardState; 16]; 16],
    food: Option<(usize, usize)>,
}

impl SnakeBoard {
    pub(crate) fn new() -> Self {
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

    pub(crate) fn advance(&mut self) {
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

pub struct SnakeSystem;

impl SnakeSystem {
    pub fn new() -> impl Runnable { Self::system(SnakeBoard::new(), SystemTime::now()) }

    fn system(mut board: SnakeBoard, mut time: SystemTime) -> impl Runnable {
        SystemBuilder::new("snake_game")
            .read_resource::<CommandManager>()
            .read_resource::<GraphicsContext>()
            .write_resource::<CanvasQueue>()
            .build(
                move |_commands, _world, (input, graphics_context, canvas_queue), _| {
                    snake_game(&mut board, &mut time, input, graphics_context, canvas_queue);

                    fn snake_game(
                        board: &mut SnakeBoard,
                        time: &mut SystemTime,
                        input: &input::CommandManager,
                        graphics_context: &GraphicsContext,
                        canvas_queue: &mut CanvasQueue,
                    ) {
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
                                canvas_queue.draw_rect(
                                    RectangleDescriptor::AnchorRect {
                                        anchor: AnchorPoint::TopLeft,
                                        position: ScreenVector::new_relative(0.5, 0.5),
                                        dimensions: ScreenVector::new_relative_to_width(
                                            0.025, 0.025,
                                        ),
                                        offset: ScreenVector::new_relative_to_width(
                                            (j as f32 - 8.0) / 16.0 / 2.0,
                                            (i as f32 - 8.0) / 16.0 / 2.0,
                                        ),
                                    },
                                    match square {
                                        BoardState::Empty => {
                                            cgmath::Vector4::new(0.1, 0.1, 0.1, 1.0)
                                        }
                                        BoardState::Snake => {
                                            cgmath::Vector4::new(0.8, 0.2, 0.2, 1.0)
                                        }
                                        BoardState::Food => {
                                            cgmath::Vector4::new(0.8, 0.8, 0.2, 1.0)
                                        }
                                        BoardState::Wall => {
                                            cgmath::Vector4::new(0.2, 0.2, 0.2, 1.0)
                                        }
                                    },
                                    graphics_context.window_size,
                                );
                            }
                        }
                    }
                },
            )
    }
}
