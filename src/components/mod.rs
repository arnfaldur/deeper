#![allow(dead_code)]

use std::f32::consts::PI;

use cgmath::Vector2;
use legion::Entity;

// Note(Jökull): Begin entity pointers
pub struct Player {
    pub model: Entity,
    pub player: Entity,
}

pub struct ActiveCamera {
    pub entity: Entity,
}

pub struct PlayerCamera {
    pub entity: Entity,
}

// end entity pointers

pub struct Agent;

pub struct AIFollow {
    pub target: Entity,
    pub minimum_distance: f32,
}

pub struct Destination {
    pub goal: Vector2<f32>,
    pub next: Vector2<f32>,
}

impl Destination {
    pub fn simple(goal: Vector2<f32>) -> Destination {
        Destination {
            goal,
            next: Vector2 { x: 0., y: 0. },
        }
    }
}

#[derive(Eq, PartialEq)]
#[derive(Copy, Clone)]
pub enum Faction {
    Enemies,
    Friends,
}

pub struct HitPoints {
    pub max: f32,
    pub health: f32,
}

#[derive(Copy, Clone)]
pub enum MapTransition {
    None,
    Deeper, // Down to the next floor
}

pub struct MapSwitcher(pub MapTransition);

pub struct Target(pub Entity);

pub struct SphericalOffset {
    pub phi: f32,
    pub theta: f32,
    pub radius: f32,
    pub theta_delta: f32,
    pub phi_delta: f32,
    pub radius_delta: f32,
}

impl SphericalOffset {
    pub fn camera_offset() -> Self {
        Self {
            phi: 0.2 * PI,
            theta: PI / 3.0,
            radius: 15.0,
            // TODO: Not satisfactory, but need to limit untraceable magic constants
            theta_delta: -0.005,
            phi_delta: 0.0025,
            radius_delta: 0.3,
        }
    }
}

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum TileType {
    Wall(Option<WallDirection>),
    Floor,
    Path,
    Nothing,
    LadderDown,
}

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum WallDirection {
    North,
    West,
    South,
    East,
}

pub struct FloorNumber(pub i32);
