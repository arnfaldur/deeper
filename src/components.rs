use specs::prelude::*;
use specs::{Component, VecStorage};
use raylib::prelude::*;

#[derive(Component, Debug, Copy, Clone)]
#[storage(VecStorage)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

#[derive(Component)]
#[storage(VecStorage)]
pub struct Agent;

pub(crate) struct MovementSystem;
impl<'a> System<'a> for MovementSystem {
    type SystemData = (WriteStorage<'a, Position>, ReadStorage<'a, Velocity>);

    fn run(&mut self, (mut pos, vel): Self::SystemData) {
        for (pos, vel) in (&mut pos, &vel).join() {
            pos.x += vel.x;
            pos.y += vel.y;
        }
    }
}

impl From<&Position> for Vector3 {
    fn from(pos: &Position) -> Vector3 {
        Vector3::new(pos.x, pos.y, 0.0)
    }
}

impl Position {
    pub fn to_vec3(self) -> Vector3 {
        Vector3::new(self.x, self.y, 0.0)
    }
}

impl From<&Position> for Vector2 {
    fn from(pos: &Position) -> Vector2 {
        Vector2::new(pos.x, pos.y)
    }
}

impl From<&Velocity> for Vector2 {
    fn from(pos: &Velocity) -> Vector2 {
        Vector2::new(pos.x, pos.y)
    }
}
