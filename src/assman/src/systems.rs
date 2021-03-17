use entity_smith::Smith;
use graphics::components::{Model3D, StaticModel};
use graphics::{GraphicsContext, GraphicsResources};
use itertools::Itertools;
use legion::systems::{ParallelRunnable, Schedule};
use legion::{Entity, IntoQuery, SystemBuilder};
use world_gen::components::{DynamicModelRequest, StaticModelRequest};

use crate::optimizer::StaticMeshOptimizer;
use crate::{AssetStore, GraphicsAssetManager};

pub fn assman_system_schedule() -> Schedule {
    Schedule::builder()
        .add_system(assman_process_dynamic_model_requests())
        .add_system(assman_process_static_model_requests())
        .build()
}

fn assman_process_dynamic_model_requests() -> impl ParallelRunnable {
    SystemBuilder::new("process_dynamic_model_requests")
        .write_component::<DynamicModelRequest>()
        .write_component::<Model3D>()
        .read_resource::<AssetStore>()
        .with_query(<(Entity, &mut DynamicModelRequest)>::query())
        .build(move |command_buffer, world, asset_store, query| {
            query.for_each_mut(world, |(entity, request)| {
                let request: &mut DynamicModelRequest = request;
                if let Some(idx) = asset_store.get_model_index(&request.label) {
                    command_buffer
                        .forge(*entity)
                        .add_component(
                            Model3D::from_index(idx)
                                .with_material(request.material)
                                .with_scale(request.scale),
                        )
                        .remove_component::<DynamicModelRequest>();
                }
            })
        })
}

fn assman_process_static_model_requests() -> impl ParallelRunnable {
    SystemBuilder::new("process_static_model_requests")
        .write_component::<StaticModelRequest>()
        .write_component::<StaticModel>()
        .write_resource::<AssetStore>()
        .write_resource::<GraphicsResources>()
        .write_resource::<GraphicsContext>()
        .with_query(<(Entity, &mut StaticModelRequest)>::query())
        .build(
            move |command_buffer,
                  world,
                  (asset_store, graphics_resources, graphics_context),
                  query| {
                let mut optimizer = StaticMeshOptimizer::new();

                query.for_each_mut(world, |(entity, request)| {
                    let request: &mut StaticModelRequest = request;
                    if let Some(idx) = asset_store.get_model_index(&request.label) {
                        optimizer.insert(idx, request.uniforms);
                        command_buffer
                            .forge(*entity)
                            .remove_component::<StaticModelRequest>();
                    }
                });

                let optimization_result = optimizer.finish(graphics_resources);

                let mut graphics_asset_manager =
                    GraphicsAssetManager::new(asset_store, graphics_resources, graphics_context);

                let optimization_result = optimization_result
                    .iter()
                    .map(|(uniforms, vertex_lists)| {
                        (
                            uniforms,
                            graphics_asset_manager
                                .allocate_graphics_model_from_vertex_lists(vertex_lists.clone()),
                        )
                    })
                    .collect_vec();

                drop(graphics_asset_manager);

                for (local_uniforms, idx) in optimization_result {
                    command_buffer
                        .smith()
                        .add_component(StaticModel::from_uniforms(
                            graphics_context,
                            idx,
                            *local_uniforms,
                        ));
                }
            },
        )
}
