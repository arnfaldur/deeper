pub use components::*;
pub use systems::TransformBuilderExtender;

pub use crate::entity_smith::TransformEntitySmith;

pub mod components;
mod entity_smith;
mod systems;

// #[derive(Default)]
// struct SceneGraph {
//     entity: Option<Entity>,
//     //changed: bool,
//     //parent: Weak<SceneNode>, // Weak pointers probably won't cut it
//     children: Vec<SceneGraph>, // TODO: consider using a smallvec here
// }

// #[derive(Default)]
// struct Ancestry {
//     pub tree: HashMap<Entity, Node>,
// }
// impl Ancestry {
//     pub fn first_gen(self, ent: Entity) -> bool {
//         self.tree
//             .get(&ent)
//             .map_or(false, |Node { parent, .. }| parent.is_none())
//     }
//     pub fn get_parent(&self, entity: Entity) -> Option<Entity> {
//         self.tree
//             .get(&entity)
//             .map(|Node { parent, .. }| *parent)
//             .flatten()
//     }
//     pub fn get_children(&self, entity: Entity) -> Option<&Vec<Entity>> {
//         self.tree.get(&entity).map(|Node { children, .. }| children)
//     }
// }
