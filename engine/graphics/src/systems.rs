use legion::systems::Runnable;
use legion::{IntoQuery, SystemBuilder};
use transforms::{Position, Transform};
use winit::window::Window;

use crate::canvas::{CanvasQueue, CanvasRenderPipeline};
use crate::components::{ActiveCamera, Camera, DynamicModel, StaticModel, Target};
use crate::data::{LocalUniforms, Material};
use crate::debug::DebugTimer;
use crate::gui::GuiRenderPipeline;
use crate::models::{ModelQueue, ModelRenderPipeline};
use crate::{GraphicsContext, GraphicsResources};

pub trait RenderBuilderExtender {
    fn add_render_systems(&mut self) -> &mut Self;
}

pub const DISPLAY_DEBUG_DEFAULT: bool = false;

pub fn render_system_schedule() -> legion::systems::Schedule {
    legion::systems::Schedule::builder()
        .add_thread_local(update_camera_system())
        .add_thread_local(render_draw_static_models_system())
        .add_thread_local(render_draw_models_system())
        .add_thread_local(render_system())
        .build()
}

fn update_camera_system() -> impl Runnable {
    SystemBuilder::new("update_camera")
        .read_component::<Camera>()
        .read_component::<Position>()
        .read_component::<Transform>()
        .read_component::<Target>()
        .read_resource::<ActiveCamera>()
        .read_resource::<GraphicsContext>()
        .write_resource::<ModelRenderPipeline>()
        .build(
            move |_, world, (active_cam, graphics_context, model_render_pass), _| {
                if let Ok((cam, cam_pos, target)) =
                    <(&Camera, &Transform, &Target)>::query().get(world, active_cam.entity)
                {
                    if let Ok(target_pos) = <&Transform>::query().get(world, target.entity) {
                        model_render_pass.set_camera(
                            graphics_context,
                            cam,
                            cam_pos.absolute.w.truncate(),
                            target_pos.absolute.w.truncate(),
                        );
                    }
                }
            },
        )
}

fn render_draw_models_system() -> impl Runnable {
    SystemBuilder::new("render_draw_models")
        .read_component::<DynamicModel>()
        .read_component::<Transform>()
        .write_resource::<ModelQueue>()
        .with_query(<(&DynamicModel, &Transform)>::query())
        .build(move |_, world, model_queue, query| {
            query.for_each_mut(world, |(model, transform)| {
                draw_model(model, transform, model_queue);
            });
        })
}

fn draw_model(model: &DynamicModel, transform: &Transform, model_queue: &mut ModelQueue) {
    model_queue.push_model(
        model.clone(),
        LocalUniforms::new(transform.absolute.into(), Material::default()),
    )
}

fn render_draw_static_models_system() -> impl Runnable {
    SystemBuilder::new("render_draw_static_models_system")
        .read_component::<StaticModel>()
        .write_resource::<ModelQueue>()
        .with_query(<&StaticModel>::query())
        .build(move |_, world, model_queue, query| {
            let for_query = world;
            query.for_each_mut(for_query, |components| {
                render_draw_static_models(components, model_queue);
            });
        })
}

fn render_draw_static_models(model: &StaticModel, model_queue: &mut ModelQueue) {
    model_queue.push_static_model(model.clone());
}

fn render_system() -> impl Runnable {
    SystemBuilder::new("render_models_system")
        .read_resource::<Window>()
        .read_resource::<GraphicsResources>()
        .read_resource::<GraphicsContext>()
        .read_resource::<ModelRenderPipeline>()
        .write_resource::<CanvasRenderPipeline>()
        .write_resource::<GuiRenderPipeline>()
        .write_resource::<ModelQueue>()
        .write_resource::<CanvasQueue>()
        .write_resource::<DebugTimer>()
        .build(
            move |_,
                  _,
                  (
                window,
                graphics_resources,
                graphics_context,
                model_render_pipeline,
                canvas_render_pipeline,
                gui_render_pipeline,
                model_queue,
                canvas_queue,
                debug_timer,
            ),
                  _| {
                render(
                    window,
                    graphics_resources,
                    graphics_context,
                    model_render_pipeline,
                    canvas_render_pipeline,
                    gui_render_pipeline,
                    model_queue,
                    canvas_queue,
                    debug_timer,
                )
            },
        )
}

fn render(
    window: &Window,
    graphics_resources: &GraphicsResources,
    graphics_context: &GraphicsContext,
    model_render_pipeline: &ModelRenderPipeline,
    canvas_render_pipeline: &mut CanvasRenderPipeline,
    gui_render_pipeline: &mut GuiRenderPipeline,
    model_queue: &mut ModelQueue,
    canvas_queue: &mut CanvasQueue,
    debug_timer: &mut DebugTimer,
) {
    let render_context = graphics_context.begin_render();

    model_render_pipeline.render(
        &render_context,
        graphics_resources,
        model_queue,
        debug_timer,
    );

    debug_timer.push("Canvas Render");

    canvas_render_pipeline.render(&render_context, canvas_queue);

    debug_timer.pop();

    gui_render_pipeline.debug_render(window, &render_context, Some(debug_timer.finish()));

    model_queue.clear();
    canvas_queue.clear();
}
