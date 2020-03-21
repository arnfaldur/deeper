extern crate specs;

use specs::{
    WorldExt,
    Builder,
    System,
    Component,
    VecStorage,
};

use specs::prelude::*;
use raylib::prelude::*;


#[derive(Component, Debug, Copy, Clone)]
#[storage(VecStorage)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

impl Position {
    pub fn new() -> Position { Position { x: 0.0, y: 0.0 } }
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

impl Velocity {
    pub fn new() -> Velocity { Velocity { x: 0.0, y: 0.0 } }
}

#[derive(Component)]
#[storage(VecStorage)]
pub struct Agent;

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

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Camera {
    pub fov: f32,
    pub up: Vector3,
}

#[derive(Component)]
pub struct Target(pub Entity);

#[derive(Component)]
pub struct Position3D(pub Vector3);

#[derive(Component)]
pub struct ActiveCamera(pub Entity);

#[derive(Component)]
pub struct SphericalOffset {
    pub theta: f32,
    pub phi: f32,
    pub radius: f32,
}

#[derive(Component)]
pub struct Model3D {
    pub idx: usize,
    pub offset: Vector3,
    pub scale: f32,
    pub z_rotation : f32,
    pub tint: Color,
}

// Note(Jökull): Probably not great to have both constructor and builder patterns
impl Model3D {
    pub fn new() -> Self { Self { idx: 0, offset: Vector3::zero(), tint: Color::WHITE, scale: 1.0, z_rotation: 0.0} }
    pub fn from_index(index: usize) -> Model3D { let mut m = Self::new(); m.idx = index; return m; }
    pub fn with_offset(mut self, offset: Vector3) -> Model3D { self.offset = offset; self }
    pub fn with_scale(mut self, scale: f32) -> Self { self.scale = scale; self }
    pub fn with_z_rotation(mut self, z_rotation: f32) -> Self { self.z_rotation = z_rotation; self }
    pub fn with_tint(mut self, tint: Color) -> Self { self.tint = tint; self }
}
#[derive(Component)]
pub struct WallTile;

#[derive(Component)]
pub struct FloorTile;


pub(crate) fn register_components(world: &mut World) {
    world.register::<Position>();
    world.register::<Position3D>();
    world.register::<Velocity>();
    world.register::<Player>();
    world.register::<Camera>();
    world.register::<Target>();
    world.register::<ActiveCamera>();
    world.register::<SphericalOffset>();
    world.register::<Model3D>();
    world.register::<WallTile>();
    world.register::<FloorTile>();
}