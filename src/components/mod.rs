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

pub struct PlayerCamera {
    pub entity: Entity,
}

// end entity pointers

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

pub struct HitPoints {
    pub max: f32,
    pub health: f32,
}
