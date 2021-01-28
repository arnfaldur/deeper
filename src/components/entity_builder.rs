use legion::query::{FilterResult, LayoutFilter};
use legion::storage::{
    ArchetypeSource, ArchetypeWriter, ComponentSource, ComponentTypeId, EntityLayout,
};

use legion::*;

use crate::components::*;



pub struct EntityBuilder {
    position: Option<Position>,
    speed: Option<Speed>,
    acceleration: Option<Acceleration>,
    orientation: Option<Orientation>,
    velocity: Option<Velocity>,
    dynamic_body: Option<DynamicBody>,
    circle_collider: Option<CircleCollider>,

    exists_spatially: bool,
}

impl EntityBuilder {
    pub fn build(&mut self, world: &mut world::World) {}
    pub fn position(mut self, pos: Vector2<f32>) -> Self {
        self.position = Some(Position(pos));
        return self;
    }
    pub fn velocity(mut self, vel: Vector2<f32>) -> Self {
        self.velocity = Some(Velocity(vel));
        return self;
    }
    pub fn dynamic_body(mut self, mass: f32) -> Self {
        self.dynamic_body = Some(DynamicBody { mass });
        return self;
        // Speed(5.),
        // Acceleration(30.),
        // Orientation(Deg(0.0)),
        // Velocity::new(),
        // CircleCollider { radius: 0.3 },
    }
    pub fn agent(mut self, accel: Acceleration) -> Self {
        self.acceleration = Some(accel);
        return self;
    }
}

impl Default for EntityBuilder {
    fn default() -> Self {
        EntityBuilder {
            position: None,
            speed: None,
            acceleration: None,
            orientation: None,
            velocity: None,
            dynamic_body: None,
            circle_collider: None,

            exists_spatially: false,
        }
    }
}

impl LayoutFilter for EntityBuilder {
    fn matches_layout(&self, components: &[ComponentTypeId]) -> FilterResult {
        FilterResult::Match(true)
    }
}

impl ArchetypeSource for EntityBuilder {
    type Filter = Self;

    fn filter(&self) -> Self::Filter {
        unimplemented!()
    }

    fn layout(&mut self) -> EntityLayout {
        unimplemented!()
    }
}

impl ComponentSource for EntityBuilder {
    fn push_components<'a>(
        &mut self,
        writer: &mut ArchetypeWriter<'a>,
        entities: impl Iterator<Item = Entity>,
    ) {
        unimplemented!()
    }
}
