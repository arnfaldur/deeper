use std::sync::Arc;

use cgmath::{Matrix4, Vector3};
use entity_smith::EntitySmith;
use wgpu::util::DeviceExt;

use crate::data::{LocalUniforms, Material};
use crate::models::ModelRenderPipeline;
use crate::{GraphicsContext, ModelID};

pub trait TemporaryModel3DEntitySmith {
    fn model(&mut self, model: Model3D) -> &mut Self;
}

impl<'a> TemporaryModel3DEntitySmith for EntitySmith<'a> {
    fn model(&mut self, model: Model3D) -> &mut Self { self.add_component(model) }
}

pub struct Camera {
    pub fov: f32,
    pub up: Vector3<f32>,
    pub roaming: bool,
}

#[derive(Clone)]
pub struct Model3D {
    pub idx: ModelID,
    pub bind_group: Arc<wgpu::BindGroup>,
    pub buffer: Arc<wgpu::Buffer>,
}

// Note(Jökull): Probably not great to have both constructor and builder patterns
impl Model3D {
    pub fn from_index(
        idx: ModelID,
        graphics_context: &GraphicsContext,
        model_render_pass: &ModelRenderPipeline,
    ) -> Self {
        let buffer = Arc::new(
            graphics_context
                .device
                .create_buffer(&wgpu::BufferDescriptor {
                    label: None,
                    size: std::mem::size_of::<LocalUniforms>() as u64,
                    usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
                    mapped_at_creation: false,
                }),
        );

        let bind_group = Arc::new(graphics_context.device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: None,
                layout: &model_render_pass.local_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &buffer,
                        offset: 0,
                        size: None,
                    },
                }],
            },
        ));
        Self {
            idx,
            bind_group,
            buffer,
        }
    }
}

#[derive(Clone)]
pub struct StaticModel {
    pub idx: ModelID,
    pub bind_group: Arc<wgpu::BindGroup>,
}

impl StaticModel {
    pub fn new(
        idx: ModelID,
        offset: Vector3<f32>,
        scale: f32,
        z_rotation: f32,
        material: Material,
        graphics_context: &GraphicsContext,
        model_render_pass: &ModelRenderPipeline,
    ) -> Self {
        let matrix = Matrix4::from_translation(offset)
            * Matrix4::from_angle_z(cgmath::Deg(z_rotation))
            * Matrix4::from_scale(scale);

        let local_uniforms = LocalUniforms::new(matrix.into(), material);

        Self::from_uniforms(idx, local_uniforms, graphics_context, model_render_pass)
    }

    pub fn from_uniforms(
        idx: ModelID,
        local_uniforms: crate::data::LocalUniforms,
        graphics_context: &GraphicsContext,
        model_render_pass: &ModelRenderPipeline,
    ) -> Self {
        let buffer =
            graphics_context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::bytes_of(&local_uniforms),
                    usage: wgpu::BufferUsage::UNIFORM,
                });

        let bind_group = Arc::new(graphics_context.device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: None,
                layout: &model_render_pass.local_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &buffer,
                        offset: 0,
                        size: None,
                    },
                }],
            },
        ));

        Self { idx, bind_group }
    }
}
