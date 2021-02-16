use legion::storage::Component;
use legion::systems::CommandBuffer;

use crate::components::*;

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
    pub fn from_entity(buffer: &'a mut legion::systems::CommandBuffer, entity: Entity) -> Self {
        Self { entity, buffer }
    }
    pub fn another(&mut self) -> &mut Self {
        self.entity = self.buffer.push(());
        return self;
    }
    pub fn build(&self) -> Entity { self.entity }
    fn add_component<T: Component>(&mut self, component: T) -> &mut Self {
        self.buffer.add_component(self.entity, component);
        return self;
    }
    pub fn position(&mut self, pos: Vector2<f32>) -> &mut Self { self.add_component(Position(pos)) }
    pub fn velocity(&mut self, vel: Vector2<f32>) -> &mut Self { self.add_component(Velocity(vel)) }
    pub fn orientation(&mut self, ori: f32) -> &mut Self {
        self.add_component(Orientation(Deg(ori)))
    }
    pub fn dynamic_body(&mut self, mass: f32) -> &mut Self {
        self.add_component(DynamicBody { mass })
    }
    pub fn circle_collider(&mut self, radius: f32) -> &mut Self {
        self.add_component(CircleCollider { radius })
    }
    pub fn model(&mut self, model: Model3D) -> &mut Self { self.add_component(model) }
    pub fn agent(&mut self, speed: f32, acceleration: f32) -> &mut Self {
        self.add_component(Speed(speed));
        self.add_component(Acceleration(acceleration))
    }
    #[deprecated(note = "builder method not implemented for a component class.")]
    pub fn any<T: Component>(&mut self, component: T) -> &mut Self { self.add_component(component) }
}
