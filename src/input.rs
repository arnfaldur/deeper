use cgmath::Vector2;
use winit::event::{Event, MouseButton, ElementState, VirtualKeyCode, MouseScrollDelta};
use std::borrow::BorrowMut;


#[derive(Default)]
pub struct EventBucket<'a>(pub Vec<Event<'a, ()>>);

pub struct ButtonState {
    pub pressed: bool,
    pub down: bool,
}

impl ButtonState {
    pub fn new() -> Self {
        Self { pressed: false, down: false }
    }
}

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
            left:   ButtonState::new(),
            right:  ButtonState::new(),
            middle: ButtonState::new(),

            pos:      Vector2::new(0.0, 0.0),
            last_pos: Vector2::new(0.0, 0.0),

            scroll: 0.0,
        }
    }

    pub fn delta(&self) -> Vector2<f32> {
        return self.pos - self.last_pos;
    }

    pub fn update_from_mouse_button(&mut self, mouse_button: &MouseButton, state: &ElementState) {
        match state {
            ElementState::Pressed => {
                match mouse_button {
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
                }
            }
            ElementState::Released => {
                match mouse_button {
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
                }
            }
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
        let mut keyboard = std::collections::HashMap::new();

        Self { mouse: MouseState::new(), keyboard }
    }

    pub fn is_key_down(&self, key: Key) -> bool {
        return match self.keyboard.get(&key) {
            Some(state) => state.down,
            None => false,
        }
    }

    pub fn is_key_pressed(&self, key: Key) -> bool {
        return match self.keyboard.get(&key) {
            Some(state) => state.pressed,
            None => false,
        }
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

        for (x, y) in &mut self.keyboard {
            y.pressed = false;
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
                        },
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
            MouseWheel { delta, .. } => {
                match delta {
                    MouseScrollDelta::LineDelta(lines, rows) => {
                        self.mouse.scroll = *rows;
                    },
                    MouseScrollDelta::PixelDelta(pixels) => {
                        self.mouse.scroll = pixels.y as f32;
                    }
                }
            },
            MouseInput { button, state, .. } => {
                self.mouse.update_from_mouse_button(button, state);
            },
            CursorMoved{ position, .. } => {
                self.mouse.pos = Vector2::new(position.x as f32, position.y as f32);
            }
            _ => ()
        }
    }
}