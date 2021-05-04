use cgmath::{Vector2, Vector3};
use entity_smith::EntitySmith;
use legion::Entity;

use crate::{Children, Parent, Position, Rotation, Transform};

pub trait TransformEntitySmith {
    fn transform_identity(&mut self) -> &mut Self;
    fn position(&mut self, pos: Vector3<f32>) -> &mut Self;
    fn pos(&mut self, pos: Vector2<f32>) -> &mut Self;
    fn orientation(&mut self, ori: f32) -> &mut Self;

    fn adopt_child(&mut self, child: Entity) -> &mut Self;
    fn child_of(&mut self, parent: Entity) -> &mut Self;
}

impl<'a> TransformEntitySmith for EntitySmith<'a> {
    fn transform_identity(&mut self) -> &mut Self { self.add_component(Transform::identity()) }
    fn position(&mut self, pos: Vector3<f32>) -> &mut Self { self.add_component(Position(pos)) }
    fn pos(&mut self, pos: Vector2<f32>) -> &mut Self {
        self.add_component(Position(pos.extend(0.)))
    }
    fn orientation(&mut self, ori: f32) -> &mut Self { self.add_component(Rotation::from_deg(ori)) }

    fn adopt_child(&mut self, child: Entity) -> &mut Self {
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
    fn child_of(&mut self, parent: Entity) -> &mut Self { self.add_component(Parent(parent)) }
}
