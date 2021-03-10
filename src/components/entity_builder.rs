use legion::storage::Component;
use legion::systems::CommandBuffer;
use legion::world::SubWorld;
use legion::{EntityStore, World};

use crate::components::*;
use crate::transform::{Position, Rotation};

pub struct EntitySmith<'a, I> {
    entity: legion::Entity,
    interface: &'a mut I,
}
// enum Interface<'a> {
//     Cmd(&'a mut legion::systems::CommandBuffer),
//     World(&'a mut legion::World),
//     SubWorld(&'a mut legion::SubWorld),
// }
// impl<Interface: Interfaces> Forge for Interface {
//     fn forge<'a>(&mut self, entity: Entity) -> EntitySmith<'a, Interface> {
//         EntitySmith {
//             entity,
//             interface: self,
//         }
//     }
// }

pub(crate) trait Forge {
    fn forge(&mut self, entity: Entity) -> EntitySmith<Self>
    where
        Self: Sized;
}

impl Forge for CommandBuffer {
    fn forge<'a>(&'a mut self, entity: Entity) -> EntitySmith<'a, CommandBuffer> {
        EntitySmith {
            entity,
            interface: self,
        }
    }
}
// impl<'a> Forge for World {
//     fn forge(&'a mut self, entity: Entity) -> EntitySmith<'a, World> {
//         EntitySmith {
//             entity,
//             interface: self,
//         }
//     }
// }
// impl<'a> Forge for SubWorld<'_> {
//     fn forge(&'a mut self, entity: Entity) -> EntitySmith<'a, SubWorld> {
//         EntitySmith {
//             entity,
//             interface: self,
//         }
//     }
// }

pub(crate) trait Smith {
    fn smith<'a>(&'a mut self) -> EntitySmith<'a, Self>
    where
        Self: Sized;
}

impl Smith for CommandBuffer {
    fn smith<'a>(&'a mut self) -> EntitySmith<'a, CommandBuffer> {
        EntitySmith {
            entity: self.push(()),
            interface: self,
        }
    }
}

// impl<'a> Smith for World {
//     fn smith(&'a mut self) -> EntitySmith<'a, World> {
//         EntitySmith {
//             entity: self.push(()),
//             interface: self,
//         }
//     }
// }
pub trait Interfaces {
    fn inter_add_component<T: Component>(&mut self, entity: Entity, component: T);
    fn add_entity(&mut self) -> Entity;
}

impl Interfaces for CommandBuffer {
    fn inter_add_component<T: Component>(&mut self, entity: Entity, component: T) {
        self.add_component(entity, component);
    }
    fn add_entity(&mut self) -> Entity { self.push(()) }
}

impl Interfaces for World {
    fn inter_add_component<T: Component>(&mut self, entity: Entity, component: T) {
        self.entry(entity).unwrap().add_component(component);
    }
    fn add_entity(&mut self) -> Entity { self.push(()) }
}
// impl Interfaces for SubWorld<'_> {
//     fn inter_add_component<T: Component>(&mut self, entity: Entity, component: T) {
//         self.entry_mut(entity).unwrap().;
//     }
// }

// impl<'a> EntitySmith<'a, CommandBuffer> {
//     pub fn another(&mut self) -> &mut Self {
//         self.entity = self.interface.push(());
//         return self;
//     }
// }
// impl<'a> EntitySmith<'a, World> {
//     pub fn another(&mut self) -> &mut Self {
//         self.entity = self.interface.push(());
//         return self;
//     }
// }

impl<'a, I: Interfaces> EntitySmith<'a, I> {
    pub fn another(&mut self) -> &mut Self {
        self.entity = self.interface.add_entity();
        return self;
    }
    fn add_component<T: Component>(&mut self, component: T) -> &mut Self {
        self.interface.inter_add_component(self.entity, component);
        return self;
    }
    // pub fn another(&mut self) -> &mut Self {
    //     self.entity = self.interface.push(());
    //     return self;
    // }
    pub fn done(&self) {}
    pub fn get_entity(&self) -> Entity { self.entity }
    pub fn craft(self) -> EntitySmith<'a, I> { self }

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
