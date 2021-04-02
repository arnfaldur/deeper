use entity_smith::Smith;
use graphics::components::{DynamicModel, StaticModel};
use graphics::models::ModelRenderPipeline;
use graphics::{GraphicsContext, GraphicsResources};
use input::{Command, CommandManager};
use itertools::Itertools;
use legion::systems::ParallelRunnable;
use legion::{Entity, IntoQuery, SystemBuilder};

use crate::components::{DynamicModelRequest, StaticModelRequest};
use crate::optimizer::StaticMeshOptimizer;
use crate::{AssetStore, GraphicsAssetManager};

pub trait AssetManagerBuilderExtender {
    fn add_assman_systems(&mut self) -> &mut Self;
}

impl AssetManagerBuilderExtender for legion::systems::Builder {
    fn add_assman_systems(&mut self) -> &mut Self {
        self.add_system(assman_process_dynamic_model_requests())
            .add_system(assman_process_static_model_requests())
            .add_system(hot_loading_system())
    }
}

fn assman_process_dynamic_model_requests() -> impl ParallelRunnable {
    SystemBuilder::new("process_dynamic_model_requests")
        .write_component::<DynamicModelRequest>()
        .write_component::<DynamicModel>()
        .read_resource::<AssetStore>()
        .read_resource::<GraphicsContext>()
        .read_resource::<ModelRenderPipeline>()
        .with_query(<(Entity, &mut DynamicModelRequest)>::query())
        .build(
            move |command_buffer,
                  world,
                  (asset_store, graphics_context, model_render_pass),
                  query| {
                query.for_each_mut(world, |(entity, request)| {
                    let request: &mut DynamicModelRequest = request;
                    if let Some(idx) = asset_store.get_model_index(&request.label) {
                        command_buffer
                            .forge(*entity)
                            .add_component(DynamicModel::from_index(
                                idx,
                                graphics_context,
                                model_render_pass,
                            ))
                            .remove_component::<DynamicModelRequest>();
                    }
                })
            },
        )
}

fn assman_process_static_model_requests() -> impl ParallelRunnable {
    SystemBuilder::new("process_static_model_requests")
        .write_component::<StaticModelRequest>()
        .write_component::<StaticModel>()
        .write_resource::<AssetStore>()
        .write_resource::<GraphicsResources>()
        .write_resource::<GraphicsContext>()
        .read_resource::<ModelRenderPipeline>()
        .with_query(<(Entity, &mut StaticModelRequest)>::query())
        .build(
            move |command_buffer,
                  world,
                  (asset_store, graphics_resources, graphics_context, model_render_pass),
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
                            idx,
                            *local_uniforms,
                            graphics_context,
                            model_render_pass,
                        ));
                }
            },
        )
}

pub fn hot_loading_system() -> impl ParallelRunnable {
    SystemBuilder::new("hot_loading_system")
        .write_resource::<AssetStore>()
        .write_resource::<GraphicsResources>()
        .write_resource::<GraphicsContext>()
        .read_resource::<CommandManager>()
        .build(
            move |_, _, (asset_store, graphics_resources, graphics_context, command_manager), _| {
                if command_manager.get(Command::DevToggleHotLoading) {
                    //asset_store.load_shaders(shaders_loaded_at, context)
                }

                if command_manager.get(Command::DevHotLoadModels) {
                    println!("Hotloading models...");
                    GraphicsAssetManager::new(asset_store, graphics_resources, graphics_context)
                        .load_models();
                }
            },
        )
}
