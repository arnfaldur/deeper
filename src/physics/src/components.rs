use cgmath::Zero;
use nphysics2d::object::{DefaultBodyHandle, DefaultColliderHandle};

#[derive(Debug)]
pub struct Velocity(pub cgmath::Vector2<f32>);

impl Default for Velocity {
    fn default() -> Self { return Velocity(cgmath::Vector2::zero()); }
}

pub struct Force(pub nphysics2d::algebra::Force2<f32>);

impl Default for Force {
    fn default() -> Self { Force(nphysics2d::algebra::Force2::zero()) }
}

pub enum Collider {
    Circle { radius: f32 },
    Square { side_length: f32 },
}

#[allow(dead_code)]
pub enum PhysicsBody {
    Disabled,
    Static,
    Dynamic { mass: f32 },
}

pub struct BodyHandle(pub DefaultBodyHandle);

pub struct ColliderHandle(pub DefaultColliderHandle);
