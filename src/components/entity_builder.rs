use legion::storage::Component;
use legion::systems::CommandBuffer;
use legion::world::SubWorld;
use legion::{EntityStore, World};

use crate::components::*;
use crate::transform::{Position, Rotation};

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
    pub fn done(&self) {}
    pub fn get_entity(&self) -> Entity { self.entity }
    pub fn craft(self) -> EntitySmith<'a> { self }
    pub fn scrap(&mut self) { self.interface.scrap(self.entity); }

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
