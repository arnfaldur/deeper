use std::time::SystemTime;

use cgmath::prelude::*;
use cgmath::{Matrix4, Vector4};
use imgui::Ui;
use legion::world::SubWorld;
use legion::*;
use nphysics2d::utils::UserData;
use wgpu::util::DeviceExt;
use wgpu::{CommandBuffer, SwapChainDescriptor, SwapChainTexture};
use zerocopy::AsBytes;

use crate::components::*;
use crate::graphics::data::LocalUniforms;
use crate::{graphics, loader};

pub struct RenderState {
    command_buffers: Vec<CommandBuffer>,
}

impl RenderState {
    pub fn new() -> Self {
        return Self {
            command_buffers: vec![],
        };
    }
}

pub trait RenderBuilderExtender {
    fn add_render_systems(&mut self) -> &mut Self;
}

impl RenderBuilderExtender for legion::systems::Builder {
    fn add_render_systems(&mut self) -> &mut Self {
        self.add_system(render_cache_local_uniforms_system())
            .add_thread_local(render_init_system())
            .add_thread_local(render_gen_global_uniforms_system())
            .add_thread_local(render_gen_local_uniforms_system(vec![]))
            .add_thread_local(render_lighting_pass_system())
            .add_thread_local(render_gui_pass_system())
            .add_thread_local(render_queue_submit_system())
    }
}

#[system]
pub fn render_init(
    #[resource] render_state: &mut RenderState,
    #[resource] context: &mut graphics::Context,
) {
    render_state.command_buffers.clear();
}

#[system]
#[write_component(Model3D)]
#[read_component(Position)]
#[read_component(Orientation)]
pub fn render_cache_local_uniforms(world: &mut SubWorld) {
    let mut model_query = <(&Position, &mut Model3D, TryRead<Orientation>)>::query();

    for (pos, mut model, rotation) in model_query.iter_mut(world) {
        let mut matrix = Matrix4::from_scale(model.scale);

        if let Some(rot) = rotation {
            matrix = Matrix4::from_angle_z(rot.0) * matrix;
        }

        matrix = Matrix4::from_translation(pos.into()) * matrix;

        model.local_uniforms_cache = LocalUniforms {
            model_matrix: matrix.into(),
            material: model.material,
        };
    }
}

#[system]
#[read_component(Model3D)]
pub fn render_gen_local_uniforms(
    world: &SubWorld,
    #[state] local_uniforms: &mut Vec<graphics::data::LocalUniforms>,
    #[resource] context: &mut graphics::Context,
    #[resource] render_state: &mut RenderState,
) {
    // Clear the vector of local_uniforms
    // This serves essentially as a memory arena that expands as needed
    local_uniforms.clear();

    let mut encoder = context
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    for (model) in <(Read<Model3D>)>::query().iter(world) {
        local_uniforms.push(model.local_uniforms_cache);
    }

    let temp_buf = context
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: local_uniforms.as_bytes(),
            usage: wgpu::BufferUsage::COPY_SRC,
        });

    for (i, (model)) in <(&Model3D)>::query().iter(world).enumerate() {
        encoder.copy_buffer_to_buffer(
            &temp_buf,
            (i * std::mem::size_of::<LocalUniforms>()) as u64,
            &model.uniform_buffer,
            0,
            std::mem::size_of::<LocalUniforms>() as u64,
        );
    }

    render_state.command_buffers.push(encoder.finish());
}

#[system]
#[read_component(Camera)]
#[read_component(Position3D)]
#[read_component(Position)]
pub fn render_gen_global_uniforms(
    world: &SubWorld,
    #[resource] context: &mut graphics::Context,
    #[resource] render_state: &mut RenderState,
    #[resource] time_started: &mut SystemTime,
    #[resource] active_cam: &ActiveCamera,
    #[resource] sc_desc: &wgpu::SwapChainDescriptor,
) {
    let mut encoder = context
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    let (cam, cam_pos, cam_target) = {
        <(&Camera, &Position3D, &Position)>::query()
            .get(world, active_cam.entity)
            .unwrap()
    };

    let proj_view_matrix = generate_view_matrix(cam, cam_pos, cam_target.into(), &sc_desc);

    let global_uniforms = graphics::data::GlobalUniforms {
        projection_view_matrix: proj_view_matrix.into(),
        eye_position: [cam_pos.0.x, cam_pos.0.y, cam_pos.0.z, 1.0],
        time: SystemTime::now()
            .duration_since(*time_started)
            .unwrap()
            .as_secs_f32(),
    };

    let new_uniform_buf = context
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: global_uniforms.as_bytes(),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_SRC,
        });

    encoder.copy_buffer_to_buffer(
        &new_uniform_buf,
        0,
        &context.uniform_buf,
        0,
        std::mem::size_of::<graphics::data::GlobalUniforms>() as u64,
    );

    render_state.command_buffers.push(encoder.finish());
}

#[system]
#[read_component(Model3D)]
#[read_component(StaticModel)]
#[read_component(Position)]
pub fn render_lighting_pass(
    world: &SubWorld,
    #[resource] context: &mut graphics::Context,
    #[resource] render_state: &mut RenderState,
    #[resource] current_frame: &wgpu::SwapChainFrame,
    #[resource] ass_man: &loader::AssetManager,
) {
    let mut encoder = context
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    {
        // initialize render pass
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &current_frame.output.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: true,
                },
            }],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                attachment: &context.depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(0),
                    store: true,
                }),
            }),
        });

        render_pass.set_pipeline(&context.pipeline);
        render_pass.set_bind_group(0, &context.bind_group, &[]);

        // render static meshes
        for model in <Read<StaticModel>>::query().iter(world) {
            render_pass.set_bind_group(1, &model.bind_group, &[]);
            for mesh in &ass_man.models[model.idx].meshes {
                render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                render_pass.draw(0..mesh.num_vertices as u32, 0..1)
            }
        }
        // render dynamic meshes
        for (_, model) in <(Read<Position>, Read<Model3D>)>::query().iter(world) {
            render_pass.set_bind_group(1, &model.bind_group, &[]);
            for mesh in &ass_man.models[model.idx].meshes {
                render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                render_pass.draw(0..mesh.num_vertices as u32, 0..1)
            }
        }
    }

    //let render_pass_buffer = encoder.finish();
    render_state.command_buffers.push(encoder.finish());
}

#[system]
pub fn render_gui_pass(
    #[resource] context: &mut graphics::Context,
    #[resource] gui_context: &mut graphics::GuiContext,
    #[resource] window: &winit::window::Window,
    #[resource] render_state: &mut RenderState,
    #[resource] current_frame: &wgpu::SwapChainFrame,
) {
    use imgui::{im_str, Condition, Window};

    gui_context
        .imgui_platform
        .prepare_frame(gui_context.imgui.io_mut(), &window)
        .expect("Failed to prepare imgui frame");

    let ui = gui_context.imgui.frame();

    {
        let window = Window::new(im_str!("Test window"));

        window
            .size([300.0, 100.0], Condition::FirstUseEver)
            .build(&ui, || {
                ui.text(im_str!("Welcome to deeper."));
                ui.separator();
                let mouse_pos = ui.io().mouse_pos;
                ui.text(im_str!(
                    "Mouse Position: ({:.1},{:.1})",
                    mouse_pos[0],
                    mouse_pos[1]
                ));
                ui.text(im_str!("FPS: ({:.2})", ui.io().framerate))
            });
    }

    let mut encoder = context
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    gui_context.imgui_platform.prepare_render(&ui, &window);

    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
            attachment: &current_frame.output.view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: true,
            },
        }],
        depth_stencil_attachment: None,
    });

    gui_context
        .imgui_renderer
        .render(ui.render(), &context.queue, &context.device, &mut rpass)
        .expect("Rendering failed");

    drop(rpass);

    render_state.command_buffers.push(encoder.finish());
}

#[system]
fn render_queue_submit(
    #[resource] context: &mut graphics::Context,
    #[resource] render_state: &mut RenderState,
) {
    context.queue.submit(render_state.command_buffers.drain(..));
}

fn generate_view_matrix(
    cam: &Camera,
    cam_pos: &Position3D,
    cam_target: cgmath::Vector3<f32>,
    sc_desc: &wgpu::SwapChainDescriptor,
) -> cgmath::Matrix4<f32> {
    let aspect_ratio = sc_desc.width as f32 / sc_desc.height as f32;

    let mx_correction = graphics::correction_matrix();

    let mx_view = cgmath::Matrix4::look_at(
        graphics::to_pos3(cam_pos.0),
        graphics::to_pos3(cam_target),
        cgmath::Vector3::unit_z(),
    );

    let mx_perspective = cgmath::perspective(cgmath::Deg(cam.fov), aspect_ratio, 1.0, 1000.0);

    mx_correction * mx_perspective * mx_view
}
