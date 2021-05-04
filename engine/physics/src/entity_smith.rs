use cgmath::Vector2;
use entity_smith::EntitySmith;
use transforms::Position;

use crate::{Collider, PhysicsBody, Velocity};

pub trait PhysicsEntitySmith {
    fn velocity(&mut self, vel: Vector2<f32>) -> &mut Self;
    fn velocity_zero(&mut self) -> &mut Self;

    fn physics_body(&mut self, body: PhysicsBody) -> &mut Self;
    fn dynamic_body(&mut self, mass: f32) -> &mut Self;
    fn static_body(&mut self) -> &mut Self;
    fn circle_collider(&mut self, radius: f32) -> &mut Self;
    fn square_collider(&mut self, side_length: f32) -> &mut Self;
    fn static_square_body(&mut self, side_length: f32) -> &mut Self;
}

impl<'a> PhysicsEntitySmith for EntitySmith<'a> {
    fn velocity(&mut self, vel: Vector2<f32>) -> &mut Self { self.add_component(Velocity(vel)) }
    fn velocity_zero(&mut self) -> &mut Self { self.add_component(Velocity::default()) }

    fn physics_body(&mut self, body: PhysicsBody) -> &mut Self { self.add_component(body) }
    fn dynamic_body(&mut self, mass: f32) -> &mut Self {
        self.ensure_component::<Position>();
        self.ensure_component::<Velocity>();
        self.add_component(PhysicsBody::Dynamic { mass })
    }
    fn static_body(&mut self) -> &mut Self { self.add_component(PhysicsBody::Static) }
    fn circle_collider(&mut self, radius: f32) -> &mut Self {
        self.add_component(Collider::Circle { radius })
    }
    fn square_collider(&mut self, side_length: f32) -> &mut Self {
        self.add_component(Collider::Square { side_length })
    }
    fn static_square_body(&mut self, side_length: f32) -> &mut Self {
        self.add_component(PhysicsBody::Static)
            .add_component(Collider::Square { side_length })
    }
}
