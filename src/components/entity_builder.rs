use std::any::Any;
use std::borrow::Borrow;
use std::ops::DerefMut;

use futures::SinkExt;
use legion::query::{FilterResult, LayoutFilter};
use legion::storage::{
    ArchetypeSource, ArchetypeWriter, Component, ComponentSource, ComponentTypeId, ComponentWriter,
    EntityLayout, IntoComponentSource, PackedStorage, UnknownComponentStorage,
};
use legion::world::SubWorld;
use legion::*;

use crate::components::*;

pub struct EntityBuilder<'a> {
    entity: Entity,
    buffer: &'a mut legion::systems::CommandBuffer,
}

impl<'a> EntityBuilder<'a> {
    pub fn from_buffer(buffer: &'a mut legion::systems::CommandBuffer) -> Self {
        Self {
            entity: buffer.push(()),
            buffer,
        }
    }
    pub fn another(&mut self) -> &mut Self {
        self.entity = self.buffer.push(());
        return self;
    }
    pub fn build(&self) -> Entity { self.entity }
    fn add_component<T: Component>(&mut self, component: T) {
        self.buffer.add_component(self.entity, component);
    }
    pub fn position(&mut self, pos: Vector2<f32>) -> &mut Self {
        self.add_component(Position(pos));
        return self;
    }
    pub fn velocity(&mut self, vel: Vector2<f32>) -> &mut Self {
        self.add_component(Velocity(vel));
        return self;
    }
    pub fn orientation(&mut self, ori: f32) -> &mut Self {
        self.add_component(Orientation(Deg(ori)));
        return self;
    }
    pub fn dynamic_body(&mut self, mass: f32) -> &mut Self {
        self.add_component(DynamicBody { mass });
        return self;
    }
    pub fn circle_collider(&mut self, radius: f32) -> &mut Self {
        self.add_component(CircleCollider{radius});
        return self;
    }
    pub fn model(&mut self, model_name: String) -> &mut Self {
        self.add_component()
        return self;
    }
    pub fn agent(&mut self, speed: f32, acceleration: f32) -> &mut Self {
        self.add_component(Speed(speed));
        self.add_component(Acceleration(acceleration));
        return self;
    }
}
