#![allow(unused)]
use cgmath::SquareMatrix;
use graphics::data::LocalUniforms;
use graphics::{GraphicsResources, ModelID};
use itertools::Itertools;

pub(crate) struct StaticMeshOptimizationEntry {
    archetype: LocalUniforms,
    instances: Vec<(ModelID, LocalUniforms)>,
}

pub(crate) struct StaticMeshOptimizer {
    entries: Vec<StaticMeshOptimizationEntry>,
}

impl StaticMeshOptimizer {
    pub(crate) fn new() -> Self { Self { entries: vec![] } }

    pub(crate) fn insert(&mut self, idx: ModelID, local_uniforms: LocalUniforms) {
        for entry in self.entries.iter_mut() {
            if entry.archetype.similar_to(&local_uniforms) {
                entry.instances.push((idx, local_uniforms));
                return;
            }
        }
        self.entries.push(StaticMeshOptimizationEntry {
            archetype: local_uniforms.with_model_matrix(cgmath::Matrix4::identity().into()),
            instances: vec![(idx, local_uniforms)],
        })
    }

    pub(crate) fn finish(
        &self,
        asset_store: &GraphicsResources,
    ) -> Vec<(LocalUniforms, graphics::data::VertexLists)> {
        self.entries
            .iter()
            .map(|entry| {
                (
                    entry.archetype,
                    entry
                        .instances
                        .iter()
                        .map(|(idx, uniforms)| {
                            asset_store
                                .models
                                .get(*idx)
                                .unwrap()
                                .vertex_lists
                                .iter()
                                .map(|vertex_list| {
                                    vertex_list
                                        .iter()
                                        .map(|vertex| vertex.transformed(uniforms.model_matrix))
                                        .collect_vec()
                                })
                                .collect_vec()
                        })
                        .collect_vec()
                        .concat(),
                )
            })
            .collect_vec()
    }
}
