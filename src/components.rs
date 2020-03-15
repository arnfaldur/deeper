use specs::prelude::*;
use specs::{Component, VecStorage};
use raylib::prelude::*;

#[derive(Component, Debug)]
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

pub struct Player;
pub struct PlayerSystem;
impl<'a> System<'a> for PlayerSystem {
    type SystemData = ();

    fn run(&mut self, data: Self::SystemData) {
        unimplemented!()
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
