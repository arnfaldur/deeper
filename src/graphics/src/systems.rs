use components::{ActiveCamera, Target};
use input::Command;
use legion::systems::Runnable;
use legion::{IntoQuery, SystemBuilder};
use transforms::{Position, Transform};

use crate::components::{Camera, Model3D, StaticModel};
use crate::gui::GuiContext;
use crate::{debug, GraphicsContext, GraphicsResources};

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
        .write_resource::<crate::GraphicsContext>()
        .build(move |_, world, (active_cam, context), _| {
            if let Ok((cam, cam_pos, target)) =
                <(&Camera, &Transform, &Target)>::query().get(world, active_cam.entity)
            {
                if let Ok(target_pos) = <&Transform>::query().get(world, target.0) {
                    context.set_3d_camera(
                        cam,
                        cam_pos.absolute.w.truncate(),
                        target_pos.absolute.w.truncate(),
                    );
                }
            }
        })
}

fn render_draw_models_system() -> impl Runnable {
    SystemBuilder::new("render_draw_models")
        .write_resource::<crate::GraphicsContext>()
        .with_query(<(&Model3D, &Transform)>::query())
        .build(move |_, world, resources, query| {
            query.for_each_mut(world, |components| {
                render_draw_models(components.0, components.1, &mut *resources);
            });
        })
}

fn render_draw_models(
    model: &Model3D,
    transform: &Transform,
    // position: &Position,
    // orientation: Option<&Rotation>,
    context: &mut crate::GraphicsContext,
) {
    context.draw_model(
        model,
        transform.absolute,
        // position.into(),
        // orientation.and(Option::from(orientation.unwrap().0)),
    );
}

fn render_draw_static_models_system() -> impl Runnable {
    SystemBuilder::new("render_draw_static_models_system")
        .write_resource::<crate::GraphicsContext>()
        .with_query(<&StaticModel>::query())
        .build(move |_, world, resources, query| {
            let for_query = world;
            query.for_each_mut(for_query, |components| {
                render_draw_static_models(components, &mut *resources);
            });
        })
}

fn render_draw_static_models(model: &StaticModel, context: &mut GraphicsContext) {
    context.draw_static_model(model.clone());
}

fn render_system() -> impl Runnable {
    SystemBuilder::new("render_system")
        .read_resource::<GraphicsResources>()
        .read_resource::<winit::window::Window>()
        .read_resource::<input::CommandManager>()
        .write_resource::<GuiContext>()
        .write_resource::<GraphicsContext>()
        .write_resource::<crate::debug::DebugTimer>()
        .build(
            move |_,
                  _,
                  (
                graphics_resources,
                window,
                command_manager,
                gui_context,
                context,
                debug_timer,
            ),
                  _| {
                render(
                    gui_context,
                    context,
                    graphics_resources,
                    window,
                    debug_timer,
                    command_manager,
                );
            },
        )
}

fn render(
    gui_context: &mut crate::gui::GuiContext,
    context: &mut crate::GraphicsContext,
    graphics_resources: &GraphicsResources,
    window: &winit::window::Window,
    debug_timer: &mut debug::DebugTimer,
    input_state: &input::CommandManager,
) {
    context.render(
        graphics_resources,
        gui_context,
        window,
        debug_timer,
        input_state.get(Command::DebugToggleInfo),
    );
}
