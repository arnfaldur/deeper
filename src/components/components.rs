extern crate specs;
extern crate cgmath;

use cgmath::{Vector2, Vector3};

use specs::{
    WorldExt,
    Builder,
    System,
    Component,
    VecStorage,
};

use specs::prelude::*;

use std::f32::consts::PI;

// Note(Jökull): Begin entity pointers
pub struct Player {
    pub entity: Entity,
    pub speed: f32,
}

impl Player {
    pub fn from_entity(entity: Entity) -> Self { return Self { entity, speed: 0.05 }; }
}

pub struct ActiveCamera(pub Entity);

pub struct PlayerCamera(pub Entity);

// end entity pointers

#[derive(Component, Debug, Copy, Clone)]
#[storage(VecStorage)]
pub struct Position(pub Vector2<f32>);

impl Position {
    pub fn new() -> Position { Position(Vector2::new(0.0, 0.0)) }
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Velocity(pub Vector2<f32>);

impl Velocity {
    pub fn new() -> Velocity { Velocity(Vector2::new(0.0, 0.0)) }
}

#[derive(Component)]
#[storage(VecStorage)]
pub struct Agent;

//impl From<&Position> for Vector3<f32> {
//    fn from(pos: &Position) -> Vector3<f32> {
//        Vector3::new(pos.0.x, pos.0.y, 0.0)
//    }
//}

impl Position {
    pub fn to_vec3(self) -> Vector3<f32> {
        Vector3::new(self.0.x, self.0.y, 0.0)
    }
}

#[derive(Component)]
pub struct StaticBody;

#[derive(Component)]
pub struct DynamicBody;

#[derive(Component)]
pub struct CircleCollider {
    pub radius: f32,
}

#[derive(Component)]
pub struct SquareCollider {
    pub side_length: f32,
}

#[derive(Component)]
pub struct AIFollow {
    pub target: Entity,
    pub minimum_distance: f32,
}

#[derive(Component)]
pub struct Camera {
    pub fov: f32,
    pub up: Vector3<f32>,
}

#[derive(Component)]
pub struct Target(pub Entity);

#[derive(Component)]
pub struct Position3D(pub Vector3<f32>);

#[derive(Component)]
pub struct SphericalOffset {
    pub theta: f32,
    pub phi: f32,
    pub radius: f32,
    pub theta_delta: f32,
    pub phi_delta: f32,
    pub radius_delta: f32,
}

// Note(Jökull): Until we have a standardized way of interacting or setting these values,
//               we can have the defaults as the most practical
impl SphericalOffset {
    pub fn new() -> Self {
        Self {
            theta: PI / 3.0,
            phi: 0.2 * PI,
            radius: 15.0,
            // TODO: Not satisfactory, but need to limit untraceable magic constants
            theta_delta: -0.005,
            phi_delta: 0.005,
            radius_delta: 0.1,
        }
    }
}

#[derive(Component)]
pub struct Model3D {
    pub idx: usize,
    pub offset: Vector3<f32>,
    pub scale: f32,
    pub z_rotation: f32,
    pub tint: Vector3<f32>,
}

// Note(Jökull): Probably not great to have both constructor and builder patterns
impl Model3D {
    pub fn new() -> Self { Self { idx: 0, offset: Vector3::new(0.0, 0.0, 0.0), tint: Vector3::new(1.0, 1.0, 1.0), scale: 1.0, z_rotation: 0.0 } }
    pub fn from_index(index: usize) -> Model3D {
        let mut m = Self::new();
        m.idx = index;
        return m;
    }
    pub fn with_offset(mut self, offset: Vector3<f32>) -> Model3D {
        self.offset = offset;
        self
    }
    pub fn with_scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }
    pub fn with_z_rotation(mut self, z_rotation: f32) -> Self {
        self.z_rotation = z_rotation;
        self
    }
    pub fn with_tint(mut self, tint: Vector3<f32>) -> Self {
        self.tint = tint;
        self
    }
}

#[derive(Component)]
pub struct WallTile;

#[derive(Component)]
pub struct FloorTile;


pub fn register_components(world: &mut World) {
    world.register::<Position>();
    world.register::<Position3D>();
    world.register::<Velocity>();
    world.register::<Camera>();
    world.register::<Target>();
    world.register::<SphericalOffset>();
    world.register::<Model3D>();
    world.register::<WallTile>();
    world.register::<FloorTile>();
    world.register::<StaticBody>();
    world.register::<DynamicBody>();
    world.register::<CircleCollider>();
    world.register::<SquareCollider>();
    world.register::<AIFollow>();
}