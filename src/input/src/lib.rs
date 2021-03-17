#![allow(unused)]

use cgmath::Vector2;
use winit::event::{ElementState, Event, MouseScrollDelta, VirtualKeyCode};

#[derive(Default)]
pub struct EventBucket<'a>(pub Vec<Event<'a, ()>>);

pub struct ButtonState {
    pub pressed: bool,
    pub down: bool,
}

impl ButtonState {
    pub fn new() -> Self {
        Self {
            pressed: false,
            down: false,
        }
    }

    fn status(&self, status: ButtonStatus) -> bool {
        match status {
            ButtonStatus::Down => self.down,
            ButtonStatus::Up => !self.down,
            ButtonStatus::Pressed => self.pressed,
            ButtonStatus::Released => !self.pressed,
        }
    }
}

#[derive(Copy, Clone)]
pub enum ButtonStatus {
    Down,
    Up,
    Pressed,
    Released,
}

type MouseButton = winit::event::MouseButton;

pub struct MouseState {
    pub left: ButtonState,
    pub right: ButtonState,
    pub middle: ButtonState,
    pub pos: Vector2<f32>,
    pub last_pos: Vector2<f32>,
    pub scroll: f32,
}

impl MouseState {
    pub fn new() -> Self {
        Self {
            left: ButtonState::new(),
            right: ButtonState::new(),
            middle: ButtonState::new(),

            pos: Vector2::new(0.0, 0.0),
            last_pos: Vector2::new(0.0, 0.0),

            scroll: 0.0,
        }
    }

    pub fn delta(&self) -> Vector2<f32> { return self.pos - self.last_pos; }

    pub fn update_from_mouse_button(&mut self, mouse_button: &MouseButton, state: &ElementState) {
        match state {
            ElementState::Pressed => match mouse_button {
                MouseButton::Left => {
                    if !self.left.down {
                        self.left.pressed = true;
                    }
                    self.left.down = true;
                }
                MouseButton::Right => {
                    if !self.right.down {
                        self.right.pressed = true;
                    }
                    self.right.down = true;
                }
                MouseButton::Middle => {
                    if !self.middle.down {
                        self.middle.pressed = true;
                    }
                    self.middle.down = true;
                }
                _ => {}
            },
            ElementState::Released => match mouse_button {
                MouseButton::Left => {
                    self.left.down = false;
                    self.left.pressed = false;
                }
                MouseButton::Right => {
                    self.right.down = false;
                    self.right.pressed = false;
                }
                MouseButton::Middle => {
                    self.middle.down = false;
                    self.middle.pressed = false;
                }
                _ => {}
            },
        }
    }
}

pub type Key = VirtualKeyCode;

pub struct InputState {
    pub mouse: MouseState,
    pub keyboard: std::collections::HashMap<Key, ButtonState>,
}

impl InputState {
    pub fn new() -> Self {
        let keyboard = std::collections::HashMap::new();

        Self {
            mouse: MouseState::new(),
            keyboard,
        }
    }

    /// Returns whether the key is currently down
    pub fn key_state(&self, key: Key, status: ButtonStatus) -> bool {
        return match self.keyboard.get(&key) {
            Some(state) => state.status(status),
            None => false,
        };
    }

    pub fn mouse_button_state(&self, mouse_button: MouseButton, status: ButtonStatus) -> bool {
        return match mouse_button {
            MouseButton::Left => self.mouse.left.status(status),
            MouseButton::Right => self.mouse.right.status(status),
            MouseButton::Middle => self.mouse.middle.status(status),
            MouseButton::Other(_) => false,
        };
    }

    // Fields that don't need re-initialization are really the exception
    // Maybe consider a less error-prone approach to loading new frame
    // (Feels like a logic bug waiting to happen)
    pub fn new_frame(&mut self) {
        self.mouse.middle.pressed = false;
        self.mouse.left.pressed = false;
        self.mouse.right.pressed = false;

        self.mouse.last_pos = self.mouse.pos;

        self.mouse.scroll = 0.0;

        for (_, state) in &mut self.keyboard {
            state.pressed = false;
        }
    }

    pub fn update_from_event(&mut self, event: &winit::event::WindowEvent) {
        use winit::event::WindowEvent::*;
        match event {
            KeyboardInput { input, .. } => {
                if let Some(key) = input.virtual_keycode {
                    let state = match self.keyboard.get_mut(&key) {
                        Some(state) => state,
                        None => {
                            self.keyboard.insert(key, ButtonState::new());
                            self.keyboard.get_mut(&key).unwrap() // Ehh..
                        }
                    };

                    match input.state {
                        ElementState::Pressed => {
                            if !state.down {
                                state.pressed = true;
                            }
                            state.down = true;
                        }
                        ElementState::Released => {
                            state.down = false;
                            state.pressed = false;
                        }
                    }
                }
            }
            MouseWheel { delta, .. } => match delta {
                MouseScrollDelta::LineDelta(_lines, rows) => {
                    self.mouse.scroll = *rows;
                }
                MouseScrollDelta::PixelDelta(pixels) => {
                    self.mouse.scroll = pixels.y as f32;
                }
            },
            MouseInput { button, state, .. } => {
                self.mouse.update_from_mouse_button(button, state);
            }
            CursorMoved { position, .. } => {
                self.mouse.pos = Vector2::new(position.x as f32, position.y as f32);
            }
            _ => (),
        }
    }
}

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
#[repr(u32)]
pub enum Command {
    DebugToggleInfo,
    DebugToggleLogic,
    DebugStepLogic,
    DebugToggleSnake,

    SnakeMoveUp,
    SnakeMoveDown,
    SnakeMoveLeft,
    SnakeMoveRight,

    DevToggleHotLoading,
    DevHotLoadModels,

    PlayerCameraMoveUp,
    PlayerCameraMoveDown,
    PlayerCameraMoveLeft,
    PlayerCameraMoveRight,

    PlayerClickToMove,
    PlayerOrbitCamera,
}

pub type KeyBinding = dyn Fn(&InputState, bool) -> bool + Send + Sync;

struct CommandState {
    logic: Box<KeyBinding>,
    state: bool,
}

impl CommandState {
    fn new(logic: Box<KeyBinding>) -> Self {
        Self {
            logic,
            state: false,
        }
    }

    fn update(&mut self, input_state: &InputState) {
        self.state = (self.logic)(input_state, self.state);
    }
}

pub struct CommandManager {
    commands: std::collections::HashMap<Command, CommandState>,
}

impl CommandManager {
    pub fn new() -> Self {
        Self {
            commands: std::collections::HashMap::new(),
        }
    }

    /// Current development keybinding
    pub fn default_bindings() -> Self {
        let mut ret = Self::new();

        ret.key_toggle(
            Command::DebugToggleInfo,
            Key::F12,
            ButtonStatus::Pressed,
            true,
            //crate::systems::rendering::DISPLAY_DEBUG_DEFAULT,
        );
        ret.simple_key_bind(Command::DebugStepLogic, Key::F10, ButtonStatus::Pressed);

        ret.key_toggle(
            Command::DebugToggleLogic,
            Key::F11,
            ButtonStatus::Pressed,
            true,
        );

        ret.advanced_bind(
            Command::DevToggleHotLoading,
            Box::new(|input_state, prev_state| {
                let input = input_state.key_state(Key::H, ButtonStatus::Pressed);
                let result = input ^ prev_state;
                if input {
                    println!("Debug shaders turned {}", if result { "ON" } else { "OFF" });
                }
                result
            }),
        );

        ret.simple_key_bind(Command::DevHotLoadModels, Key::L, ButtonStatus::Pressed);

        ret.simple_key_bind(Command::PlayerCameraMoveUp, Key::E, ButtonStatus::Pressed);
        ret.simple_key_bind(Command::PlayerCameraMoveDown, Key::D, ButtonStatus::Pressed);
        ret.simple_key_bind(Command::PlayerCameraMoveLeft, Key::S, ButtonStatus::Pressed);
        ret.simple_key_bind(
            Command::PlayerCameraMoveRight,
            Key::F,
            ButtonStatus::Pressed,
        );

        ret.simple_mouse_bind(
            Command::PlayerClickToMove,
            MouseButton::Left,
            ButtonStatus::Pressed,
        );
        ret.simple_mouse_bind(
            Command::PlayerOrbitCamera,
            MouseButton::Right,
            ButtonStatus::Down,
        );

        ret.key_toggle(
            Command::DebugToggleSnake,
            Key::P,
            ButtonStatus::Pressed,
            false,
        );

        ret.simple_key_bind(Command::SnakeMoveUp, Key::Up, ButtonStatus::Pressed);
        ret.simple_key_bind(Command::SnakeMoveDown, Key::Down, ButtonStatus::Pressed);
        ret.simple_key_bind(Command::SnakeMoveLeft, Key::Left, ButtonStatus::Pressed);
        ret.simple_key_bind(Command::SnakeMoveRight, Key::Right, ButtonStatus::Pressed);

        ret
    }

    pub fn get(&self, command: Command) -> bool {
        if let Some(command_state) = self.commands.get(&command) {
            command_state.state
        } else {
            false
        }
    }

    pub fn simple_key_bind(&mut self, command: Command, key: Key, button_status: ButtonStatus) {
        self.commands.insert(
            command,
            CommandState::new(Box::new(move |input_state: &InputState, _| {
                input_state.key_state(key, button_status)
            })),
        );
    }

    pub fn key_toggle(
        &mut self,
        command: Command,
        key: Key,
        button_status: ButtonStatus,
        default: bool,
    ) {
        let mut state = CommandState::new(Box::new(move |input_state, prev_state| {
            input_state.key_state(key, button_status) ^ prev_state
        }));

        state.state = default;

        self.commands.insert(command, state);
    }

    pub fn simple_mouse_bind(
        &mut self,
        command: Command,
        mouse_button: MouseButton,
        button_status: ButtonStatus,
    ) {
        self.commands.insert(
            command,
            CommandState::new(Box::new(move |input_state: &InputState, _| {
                input_state.mouse_button_state(mouse_button, button_status)
            })),
        );
    }

    pub fn advanced_bind(&mut self, command: Command, logic: Box<KeyBinding>) {
        self.commands.insert(command, CommandState::new(logic));
    }

    pub fn has_binding(&self, command: Command) -> bool { self.commands.contains_key(&command) }

    pub fn update(&mut self, input_state: &InputState) {
        for (_, state) in &mut self.commands {
            state.update(input_state);
        }
    }
}
