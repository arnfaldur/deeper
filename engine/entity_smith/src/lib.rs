use std::fmt::Formatter;

use legion::storage::{Component, ComponentTypeId};
use legion::systems::CommandBuffer;
use legion::Entity;

pub struct FrameTime(pub f32);

pub struct Marker;

pub struct Name(String);

impl std::fmt::Display for Name {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { self.0.fmt(f) }
}

pub struct EntitySmith<'a> {
    pub entity: legion::Entity,
    pub interface: &'a mut CommandBuffer,
}

pub trait Smith {
    fn smith(&mut self) -> EntitySmith;
    fn forge(&mut self, entity: Entity) -> EntitySmith;
    fn scrap(&mut self, entity: Entity);
}

impl Smith for CommandBuffer {
    fn smith(&mut self) -> EntitySmith {
        EntitySmith {
            entity: self.push(()),
            interface: self,
        }
    }
    fn forge(&mut self, entity: Entity) -> EntitySmith {
        EntitySmith {
            entity,
            interface: self,
        }
    }
    fn scrap(&mut self, entity: Entity) {
        // TODO: add entity reference cleanup here
        // self.exec_mut(|world,resources|{ });
        self.remove(entity);
    }
}

impl<'a> EntitySmith<'a> {
    pub fn another(&mut self) -> &mut Self {
        self.entity = self.interface.push(());
        self
    }
    pub fn add_component<T: Component>(&mut self, component: T) -> &mut Self {
        self.interface.add_component(self.entity, component);
        self
    }
    pub fn remove_component<T: Component>(&mut self) -> &mut Self {
        self.interface.remove_component::<T>(self.entity);
        self
    }
    pub fn done(&self) {}
    pub fn get_entity(&self) -> Entity { self.entity }
    pub fn craft(self) -> Self { self }
    pub fn scrap(&mut self) { self.interface.scrap(self.entity); }

    pub fn ensure_component<T: Component + Default>(&mut self) {
        let entity = self.entity;
        self.interface.exec_mut(move |world, _| {
            let mut entry = world.entry(entity).unwrap();
            if entry.get_component::<T>().is_err() {
                println!("ensuring component {}", ComponentTypeId::of::<T>());
                entry.add_component(T::default());
            }
        });
    }
    #[allow(dead_code)]
    fn prevent_component<T: Component + Default>(&mut self) {
        let entity = self.entity;
        self.interface.exec_mut(move |world, _| {
            let mut entry = world.entry(entity).unwrap();
            if entry.get_component::<T>().is_ok() {
                //println!("preventing component {}", ComponentTypeId::of::<T>());
                entry.remove_component::<T>();
            }
        });
    }

    pub fn agent(&mut self, speed: f32, acceleration: f32) -> &mut Self {
        self.add_component(Speed(speed))
            .add_component(Acceleration(acceleration))
    }

    pub fn mark(&mut self) -> &mut Self { self.add_component(Marker) }
    pub fn name(&mut self, name: &str) -> &mut Self { self.add_component(Name(String::from(name))) }
    // #[deprecated(note = "builder method not implemented for a component class.")]
    // TODO: find more elegant solution for thim
    pub fn any<T: Component>(&mut self, component: T) -> &mut Self { self.add_component(component) }
}

// TODO: move these to a better place

pub struct Speed(pub f32);

pub struct Acceleration(pub f32);
