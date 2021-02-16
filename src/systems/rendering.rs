use cgmath::prelude::*;
use legion::world::SubWorld;
use legion::*;
use zerocopy::AsBytes;

use crate::components::*;
use crate::graphics;
use crate::loader::AssetManager;

pub trait RenderBuilderExtender {
    fn add_render_systems(&mut self) -> &mut Self;
}

impl RenderBuilderExtender for legion::systems::Builder {
    fn add_render_systems(&mut self) -> &mut Self {
        self.add_thread_local(update_camera_system())
            .add_thread_local(render_draw_static_models_system())
            .add_thread_local(render_draw_models_system())
            .add_thread_local(render_gui_init_system())
            .add_thread_local(render_gui_test_system())
            .add_thread_local(render_system())
    }
}

#[system]
#[read_component(Camera)]
#[read_component(Position3D)]
#[read_component(Position)]
fn update_camera(
    world: &SubWorld,
    #[resource] context: &mut graphics::Context,
    #[resource] active_cam: &ActiveCamera,
) {
    let (cam, cam_pos, cam_target) = {
        <(&Camera, &Position3D, &Position)>::query()
            .get(world, active_cam.entity)
            .unwrap()
    };

    context.set_3d_camera(cam, cam_pos.0, cam_target.0.extend(0.0));
}

#[system(for_each)]
fn render_draw_models(
    model: &Model3D,
    position: &Position,
    orientation: Option<&Orientation>,
    #[resource] context: &mut graphics::Context,
) {
    context.draw_model(
        model.clone(),
        position.into(),
        orientation.and(Option::from(orientation.unwrap().0)),
    );
}

#[system(for_each)]
fn render_draw_static_models(model: &StaticModel, #[resource] context: &mut graphics::Context) {
    context.draw_static_model(model.clone());
}

#[system]
fn render_gui_init(
    #[resource] gui_context: &mut graphics::gui::GuiContext,
    #[resource] window: &winit::window::Window,
) {
    gui_context.prep_frame(window);
    gui_context.new_frame();
}

#[system]
fn render_gui_test(#[resource] _gui_context: &mut graphics::gui::GuiContext) {
    graphics::gui::GuiContext::with_ui(|ui| {
        use imgui::{im_str, Condition};
        let test_window = imgui::Window::new(im_str!("Test Window"));

        test_window
            .size([300.0, 100.0], Condition::FirstUseEver)
            .build(ui, || {
                ui.text("Welcome to deeper.");
            });
    });
}

#[system]
fn render(
    #[resource] gui_context: &mut graphics::gui::GuiContext,
    #[resource] context: &mut graphics::Context,
    #[resource] ass_man: &AssetManager,
    #[resource] window: &winit::window::Window,
) {
    context.render(ass_man, gui_context, window);
}
