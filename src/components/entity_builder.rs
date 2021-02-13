use std::any::Any;
use std::borrow::Borrow;
use std::ops::DerefMut;

use futures::SinkExt;
use legion::query::{FilterResult, LayoutFilter};
use legion::storage::{
    ArchetypeSource, ArchetypeWriter, Component, ComponentSource, ComponentTypeId, ComponentWriter,
    EntityLayout, IntoComponentSource, PackedStorage, UnknownComponentStorage,
};
use legion::*;

use crate::components::*;

trait MyComponent: Any + 'static + Send + Sync + Sized {}
#[derive(Default)]
pub struct EntityBuilder {
    layout: EntityLayout,
    components: Vec<(
        Box<dyn Any + 'static + Send + Sync>,
        ComponentTypeId,
        fn() -> Box<dyn UnknownComponentStorage>,
    )>,
}

impl EntityBuilder {
    pub fn new() -> Self {
        Self {
            layout: Default::default(),
            components: vec![],
        }
    }
    pub fn build(&mut self) -> Self { return std::mem::take(self); }
    fn add_component<T: Component>(&mut self, component: T) {
        if !self.layout.has_component::<T>() {
            println!("bois: {:?}", self.components);
            self.layout.register_component::<T>();
            self.components
                .push((Box::new(component), ComponentTypeId::of::<T>(), || {
                    Box::new(T::Storage::default()) as Box<dyn UnknownComponentStorage>
                }));
        }
    }
    pub fn position(&mut self, pos: Vector2<f32>) -> &mut Self {
        self.add_component::<Position>(Position(pos));
        return self;
    }
    pub fn velocity(&mut self, vel: Vector2<f32>) -> &mut Self {
        self.add_component::<Velocity>(Velocity(vel));
        return self;
    }
    pub fn dynamic_body(&mut self, mass: f32) -> &mut Self {
        self.add_component::<DynamicBody>(DynamicBody { mass });
        return self;
    }
    pub fn agent(&mut self, accel: Acceleration) -> &mut Self {
        self.add_component::<Acceleration>(accel);
        return self;
    }
}

impl ArchetypeSource for EntityBuilder {
    type Filter = EntityLayout;

    fn filter(&self) -> Self::Filter { self.layout.clone() }

    fn layout(&mut self) -> EntityLayout { self.layout.clone() }
}

impl ComponentSource for EntityBuilder {
    fn push_components<'a>(
        &mut self,
        writer: &mut ArchetypeWriter<'a>,
        mut entities: impl Iterator<Item = Entity>,
    ) {
        fn write<T: Component>(writer: &mut ArchetypeWriter, comp: &mut Box<T>) {
            let mut target = writer.claim_components::<T>();
            unsafe {
                target.extend_memcopy(**comp, 1);
            }
            std::mem::forget(comp)
        }
        for boi in &mut self.components {
            write(writer, &mut boi.0);
        }
        writer.push(entities.next().unwrap());
    }
}

impl IntoComponentSource for EntityBuilder {
    type Source = Self;

    fn into(self) -> Self::Source { self }
}

// impl IntoIterator for EntityBuilder {
//     type Item = ();
//     type IntoIter = ();
//
//     fn into_iter(self) -> Self::IntoIter { unimplemented!() }
// }
