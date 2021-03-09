use legion::storage::Component;
use legion::systems::CommandBuffer;

use crate::components::*;
use crate::transform::{Position, Rotation};

pub struct EntitySmith<'a> {
    entity: legion::Entity,
    buffer: &'a mut legion::systems::CommandBuffer,
}

impl<'a> From<&'a mut legion::systems::CommandBuffer> for EntitySmith<'a> {
    fn from(buffer: &'a mut CommandBuffer) -> Self {
        Self {
            entity: buffer.push(()),
            buffer,
        }
    }
}

impl<'a> EntitySmith<'a> {
    pub fn from_buffer(buffer: &'a mut legion::systems::CommandBuffer) -> Self {
        Self {
            entity: buffer.push(()),
            buffer,
        }
    }
    pub fn from_entity(buffer: &'a mut legion::systems::CommandBuffer, entity: Entity) -> Self {
        Self { entity, buffer }
    }
    pub fn another(&mut self) -> &mut Self {
        self.entity = self.buffer.push(());
        return self;
    }
    pub fn done(&self) {}
    pub fn get_entity(&self) -> Entity { self.entity }
    pub fn craft(self) -> EntitySmith<'a> { self }
    fn add_component<T: Component>(&mut self, component: T) -> &mut Self {
        self.buffer.add_component(self.entity, component);
        return self;
    }

    pub fn position(&mut self, pos: Vector3<f32>) -> &mut Self { self.add_component(Position(pos)) }
    pub fn velocity(&mut self, vel: Vector2<f32>) -> &mut Self { self.add_component(Velocity(vel)) }
    pub fn velocity_zero(&mut self) -> &mut Self { self.add_component(Velocity::new()) }
    pub fn orientation(&mut self, ori: f32) -> &mut Self {
        self.add_component(Rotation::from_deg(ori))
    }
    pub fn physics_body(&mut self, body: PhysicsBody) -> &mut Self { self.add_component(body) }
    pub fn dynamic_body(&mut self, mass: f32) -> &mut Self {
        self.add_component(PhysicsBody::Dynamic { mass })
    }
    pub fn circle_collider(&mut self, radius: f32) -> &mut Self {
        self.add_component(Collider::Circle { radius })
    }
    pub fn model(&mut self, model: Model3D) -> &mut Self { self.add_component(model) }
    pub fn agent(&mut self, speed: f32, acceleration: f32) -> &mut Self {
        self.add_component(Speed(speed));
        self.add_component(Acceleration(acceleration))
    }
    pub fn mark(&mut self) -> &mut Self { self.add_component(Marker) }
    pub fn name(&mut self, name: &str) -> &mut Self { self.add_component(Name(String::from(name))) }
    #[deprecated(note = "builder method not implemented for a component class.")]
    pub fn any<T: Component>(&mut self, component: T) -> &mut Self { self.add_component(component) }
}
