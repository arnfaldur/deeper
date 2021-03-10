use legion::storage::{Component, ComponentTypeId};
use legion::systems::CommandBuffer;

use crate::components::*;
use crate::physics::{Collider, PhysicsBody, Velocity};
use crate::transform::{Position, Rotation, Transform};

pub struct EntitySmith<'a> {
    entity: legion::Entity,
    interface: &'a mut CommandBuffer,
}

pub(crate) trait Smith {
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
        return self;
    }
    fn add_component<T: Component>(&mut self, component: T) -> &mut Self {
        self.interface.add_component(self.entity, component);
        return self;
    }
    pub fn remove_component<T: Component>(&mut self) -> &mut Self {
        self.interface.remove_component::<T>(self.entity);
        return self;
    }
    pub fn done(&self) {}
    pub fn get_entity(&self) -> Entity { self.entity }
    pub fn craft(self) -> Self { self }
    pub fn scrap(&mut self) { self.interface.scrap(self.entity); }

    fn ensure_component<T: Component + Default>(&mut self) {
        let entity = self.entity;
        self.interface.exec_mut(move |world, _| {
            let mut entry = world.entry(entity).unwrap();
            if entry.get_component::<T>().is_err() {
                println!("ensuring component {}", ComponentTypeId::of::<T>());
                entry.add_component(T::default());
            }
        });
    }
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

    pub fn transform_identity(&mut self) -> &mut Self { self.add_component(Transform::identity()) }
    pub fn position(&mut self, pos: Vector3<f32>) -> &mut Self { self.add_component(Position(pos)) }
    pub fn pos(&mut self, pos: Vector2<f32>) -> &mut Self {
        self.add_component(Position(pos.extend(0.)))
    }
    pub fn velocity(&mut self, vel: Vector2<f32>) -> &mut Self { self.add_component(Velocity(vel)) }
    pub fn velocity_zero(&mut self) -> &mut Self { self.add_component(Velocity::default()) }
    pub fn orientation(&mut self, ori: f32) -> &mut Self {
        self.add_component(Rotation::from_deg(ori))
    }

    pub fn adopt_child(&mut self, child: Entity) -> &mut Self {
        let me = self.entity;
        self.interface.exec_mut(move |world, _| {
            if let Some(mut entry) = world.entry(me) {
                if let Ok(children) = entry.get_component_mut::<Children>() {
                    children.0.insert(child);
                } else {
                    entry.add_component(Children([child].iter().cloned().collect()));
                }
            }
        });
        return self;
    }
    pub fn child_of(&mut self, parent: Entity) -> &mut Self { self.add_component(Parent(parent)) }

    pub fn physics_body(&mut self, body: PhysicsBody) -> &mut Self { self.add_component(body) }
    pub fn dynamic_body(&mut self, mass: f32) -> &mut Self {
        self.ensure_component::<Position>();
        self.ensure_component::<Velocity>();
        self.add_component(PhysicsBody::Dynamic { mass })
    }
    pub fn static_body(&mut self) -> &mut Self { self.add_component(PhysicsBody::Static) }
    pub fn circle_collider(&mut self, radius: f32) -> &mut Self {
        self.add_component(Collider::Circle { radius })
    }
    pub fn square_collider(&mut self, side_length: f32) -> &mut Self {
        self.add_component(Collider::Square { side_length })
    }
    pub fn static_square_body(&mut self, side_length: f32) -> &mut Self {
        self.add_component(PhysicsBody::Static)
            .add_component(Collider::Square { side_length })
    }
    pub fn model(&mut self, model: Model3D) -> &mut Self { self.add_component(model) }
    pub fn agent(&mut self, speed: f32, acceleration: f32) -> &mut Self {
        self.add_component(Speed(speed))
            .add_component(Acceleration(acceleration))
    }

    pub fn mark(&mut self) -> &mut Self { self.add_component(Marker) }
    pub fn name(&mut self, name: &str) -> &mut Self { self.add_component(Name(String::from(name))) }
    #[deprecated(note = "builder method not implemented for a component class.")]
    pub fn any<T: Component>(&mut self, component: T) -> &mut Self { self.add_component(component) }
}
