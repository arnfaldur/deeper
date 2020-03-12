use specs::prelude::*;
use specs::{Component, VecStorage};
use raylib::prelude::*;

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Position {
    x: f32,
    y: f32,
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Velocity {
    x: f32,
    y: f32,
}

#[derive(Component)]
#[storage(VecStorage)]
pub struct Agent {}

struct SystemMovement;
impl<'a> System<'a> for SystemMovement {
    type SystemData = (WriteStorage<'a, Position>, ReadStorage<'a, Velocity>);

    fn run(&mut self, (mut pos, vel): Self::SystemData) {
        for (pos, vel) in (&mut pos, &vel).join() {
            pos.x += vel.x;
            pos.y += vel.y;
        }
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