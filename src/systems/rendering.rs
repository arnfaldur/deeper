use zerocopy::AsBytes;

use cgmath::prelude::*;
use cgmath::{Matrix4, Vector4};

use crate::{loader, graphics};
use crate::components::*;

use std::time::SystemTime;

use legion::*;
use legion::world::SubWorld;

#[system]
#[read_component(Camera)]
#[read_component(Target)]
#[read_component(Position3D)]
#[read_component(Position)]
#[read_component(Orientation)]
#[read_component(Model3D)]
#[read_component(StaticModel)]
#[read_component(HitPoints)]
pub fn rendering(
    world: &SubWorld,
    commands: &mut legion::systems::CommandBuffer,
    #[state] time_started: &mut SystemTime,
    #[resource] context: &mut graphics::Context,
    #[resource] ass_man: &loader::AssetManager,
    #[resource] active_cam: &ActiveCamera,
) {
    let frame = context.swap_chain.get_next_texture()
        .expect("Failure to get next texture in swap chain.");

    let (cam, cam_pos, cam_target) = {
        <(&Camera, &Position3D, &Position)>::query().get(world, active_cam.entity).unwrap()
    };

    let proj_view_matrix = generate_view_matrix(cam, cam_pos, cam_target.into(), &context.sc_desc);

    let global_uniforms = graphics::GlobalUniforms {
        projection_view_matrix: proj_view_matrix.into(),
        eye_position: [cam_pos.0.x, cam_pos.0.y, cam_pos.0.z, 1.0],
        time: SystemTime::now().duration_since(*time_started).unwrap().as_secs_f32(),
    };

    let new_uniform_buf = context.device.create_buffer_with_data(
        global_uniforms.as_bytes(),
        wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_SRC,
    );

    let mut encoder = context.device.create_command_encoder(
        &wgpu::CommandEncoderDescriptor { label: None }
    );

    encoder.copy_buffer_to_buffer(
        &new_uniform_buf,
        0,
        &context.uniform_buf,
        0,
        std::mem::size_of::<graphics::GlobalUniforms>() as u64,
    );

    let mut local_uniforms = vec!();

    let mut query = <(&Position, &Model3D, TryRead<Orientation>, TryRead<HitPoints>)>::query();

    for (pos, model, rotation, hp) in query.iter(world) {

        let mut matrix = Matrix4::from_scale(model.scale);

        if let Some(rot) = rotation {
            matrix = Matrix4::from_angle_z(rot.0) * matrix;
        }
        matrix = Matrix4::from_translation(pos.into()) * matrix;

        let bloody_red = Vector4::unit_x() + Vector4::unit_w();

        let alb = Vector4::from(model.material.albedo);

        let mut redder_mat: graphics::Material = model.material.clone();
        if let Some(hp) = hp {
            redder_mat.albedo = bloody_red.lerp(alb, hp.health / hp.max).into();
        }

        let model_uniforms = graphics::LocalUniforms {
            model_matrix: matrix.into(),
            material: redder_mat,
        };

        local_uniforms.push(model_uniforms);
    }

    let temp_buf = context.device.create_buffer_with_data(
        local_uniforms.as_bytes(),
        wgpu::BufferUsage::COPY_SRC,
    );

    for (i, (_, model)) in <(&Position, &Model3D)>::query().iter(world).enumerate() {
        encoder.copy_buffer_to_buffer(
            &temp_buf,
            (i * std::mem::size_of::<graphics::LocalUniforms>()) as u64,
            &model.uniform_buffer,
            0,
            std::mem::size_of::<graphics::LocalUniforms>() as u64,
        );
    }

    {
        // initialize render pass
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &frame.view,
                resolve_target: None,
                load_op: wgpu::LoadOp::Clear,
                store_op: wgpu::StoreOp::Store,
                clear_color: wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 },
            }],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                attachment: &context.depth_view,
                depth_load_op: wgpu::LoadOp::Clear,
                depth_store_op: wgpu::StoreOp::Store,
                clear_depth: 1.0,
                stencil_load_op: wgpu::LoadOp::Clear,
                stencil_store_op: wgpu::StoreOp::Store,
                clear_stencil: 0,
            }),
        });

        render_pass.set_pipeline(&context.pipeline);
        render_pass.set_bind_group(0, &context.bind_group, &[]);

        // render static meshes
        for model in <(Read<StaticModel>)>::query().iter(world) {
            render_pass.set_bind_group(1, &model.bind_group, &[]);
            for mesh in &ass_man.models[model.idx].meshes {
                render_pass.set_vertex_buffer(0, &mesh.vertex_buffer, 0, 0);
                render_pass.draw(0..mesh.num_vertices as u32, 0..1)
            }
        }
        // render dynamic meshes
        for (_, model) in <(Read<Position>, Read<Model3D>)>::query().iter(world) {
            render_pass.set_bind_group(1, &model.bind_group, &[]);
            for mesh in &ass_man.models[model.idx].meshes {
                render_pass.set_vertex_buffer(0, &mesh.vertex_buffer, 0, 0);
                render_pass.draw(0..mesh.num_vertices as u32, 0..1)
            }
        }
    }

    let command_buf = encoder.finish();

    context.queue.submit(&[command_buf]);
}

fn generate_view_matrix(cam: &Camera, cam_pos: &Position3D, cam_target: cgmath::Vector3<f32>, sc_desc: &wgpu::SwapChainDescriptor) -> cgmath::Matrix4<f32> {
    let aspect_ratio = sc_desc.width as f32 / sc_desc.height as f32;

    let mx_correction = graphics::correction_matrix();

    let mx_view = cgmath::Matrix4::look_at(
        graphics::to_pos3(cam_pos.0),
        graphics::to_pos3(cam_target),
        cgmath::Vector3::unit_z(),
    );

    let mx_perspective = cgmath::perspective(
        cgmath::Deg(cam.fov),
        aspect_ratio,
        1.0,
        1000.0,
    );

    mx_correction * mx_perspective * mx_view
}
