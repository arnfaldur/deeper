#![allow(dead_code)]

use cgmath::Vector2;
use legion::Entity;

/*
    Welcome to Ms. Deeper's home for orphan components.

    Please take pity on these poor components and give them a proper home.
 */

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
    Frenemies,
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

pub struct FloorNumber(pub i32);
