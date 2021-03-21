use itertools::Itertools;
use wgpu::util::DeviceExt;

use crate::components::{Camera, DynamicModel, StaticModel};
use crate::data::{GlobalUniforms, LocalUniforms};
use crate::debug::DebugTimer;
use crate::{GraphicsContext, GraphicsResources, RenderContext, TextureID};

// TODO: Have ass_man auto-load all Shaders
//const FRAG_SRC: &str = include_str!("../../assets/Shaders/forward.frag");
//const DYNAMIC_VERT_SRC: &str = include_str!("../../assets/Shaders/forward.vert");
//const STATIC_VERT_SRC: &str = include_str!("../../assets/Shaders/static.vert");

pub struct ModelQueue {
    dynamic_models: Vec<(DynamicModel, LocalUniforms)>,
    static_models: Vec<StaticModel>,
}

impl ModelQueue {
    pub fn new() -> Self {
        Self {
            dynamic_models: vec![],
            static_models: vec![],
        }
    }

    pub fn push_static_model(&mut self, model: StaticModel) { self.static_models.push(model); }

    pub fn push_model(&mut self, model: DynamicModel, uniforms: LocalUniforms) {
        self.dynamic_models.push((model, uniforms));
    }

    pub fn clear(&mut self) {
        self.dynamic_models.clear();
        self.static_models.clear();
    }

    pub fn drain(&mut self) -> Self {
        Self {
            dynamic_models: self.dynamic_models.drain(..).collect_vec(),
            static_models: self.static_models.drain(..).collect_vec(),
        }
    }
}

pub struct ModelRenderPipeline {
    global_uniform_buf: wgpu::Buffer,
    global_bind_group: wgpu::BindGroup,
    pub(crate) local_bind_group_layout: wgpu::BindGroupLayout,
    static_pipeline: wgpu::RenderPipeline,
    dynamic_pipeline: wgpu::RenderPipeline,
    _pipeline_layout: wgpu::PipelineLayout,
    _texture_sampler: wgpu::Sampler,
}

impl ModelRenderPipeline {
    pub fn new(
        context: &GraphicsContext,
        graphics_resources: &GraphicsResources,
        color_texture_id: TextureID,
    ) -> Self {
        let device = &context.device;

        let global_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler {
                            filtering: true,
                            comparison: false,
                        },
                        count: None,
                    },
                ],
            });

        let local_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Local Bind Group Layout -- Models"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let global_uniforms: GlobalUniforms = Default::default();

        let global_uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Global Shader Uniforms"),
            contents: bytemuck::bytes_of(&global_uniforms),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let color_texture_view = &graphics_resources
            .textures
            .get(color_texture_id)
            .unwrap()
            .texture_view;

        let texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let global_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &global_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &global_uniform_buf,
                        offset: 0,
                        size: None,
                    },
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(color_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&texture_sampler),
                },
            ],
        });

        let static_vs_module = graphics_resources.shaders.get("static.vert").unwrap();
        let dynamic_vs_module = graphics_resources.shaders.get("forward.vert").unwrap();
        let fs_module = graphics_resources.shaders.get("forward.frag").unwrap();

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Model Render Pipeline Layout"),
            bind_group_layouts: &[&global_bind_group_layout, &local_bind_group_layout],
            push_constant_ranges: &[],
        });

        let static_pipeline =
            Self::compile_pipeline(&device, &pipeline_layout, &static_vs_module, &fs_module);

        let dynamic_pipeline =
            Self::compile_pipeline(&device, &pipeline_layout, &dynamic_vs_module, &fs_module);

        Self {
            global_uniform_buf,
            global_bind_group,
            local_bind_group_layout,
            static_pipeline,
            dynamic_pipeline,
            _pipeline_layout: pipeline_layout,
            _texture_sampler: texture_sampler,
        }
    }

    pub fn render(
        &self,
        render_context: &RenderContext,
        graphics_resources: &GraphicsResources,
        model_queue: &ModelQueue,
        debug_info: &mut DebugTimer,
    ) {
        debug_info.push("Model Render Pass");

        let depth_view =
            Self::create_depth_view(&render_context.device, render_context.window_size);

        debug_info.push("Static Model Render");

        let mut encoder =
            render_context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Static Model Render"),
                });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &render_context.current_frame.output.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: true,
                },
            }],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                attachment: &depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        });

        render_pass.set_pipeline(&self.static_pipeline);
        render_pass.set_bind_group(0, &self.global_bind_group, &[]);

        // render static meshes
        for model in &model_queue.static_models {
            render_pass.set_bind_group(1, &model.bind_group, &[]);
            for mesh in &graphics_resources.models[model.idx].meshes {
                render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                render_pass.draw(0..mesh.num_vertices as u32, 0..1)
            }
        }

        drop(render_pass);

        render_context
            .queue
            .submit(std::iter::once(encoder.finish()));

        debug_info.pop();

        debug_info.push("Dynamic Model Render");

        for (model, uniforms) in &model_queue.dynamic_models {
            render_context
                .queue
                .write_buffer(&model.buffer, 0, bytemuck::bytes_of(uniforms));
        }

        let mut encoder =
            render_context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Dynamic Model Render"),
                });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &render_context.current_frame.output.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            }],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                attachment: &depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                }),
                stencil_ops: None,
            }),
        });

        render_pass.set_pipeline(&self.dynamic_pipeline);
        render_pass.set_bind_group(0, &self.global_bind_group, &[]);

        // render dynamic meshes
        for (model, _) in model_queue.dynamic_models.iter() {
            render_pass.set_bind_group(1, &model.bind_group, &[]);
            for mesh in &graphics_resources.models[model.idx].meshes {
                render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                render_pass.draw(0..mesh.num_vertices as u32, 0..1)
            }
        }
        drop(render_pass);

        render_context
            .queue
            .submit(std::iter::once(encoder.finish()));

        debug_info.pop();

        debug_info.pop();
    }

    // TODO: Possibly cleaner to do just do "set view matrix"?
    pub fn set_camera(
        &mut self,
        graphics_context: &GraphicsContext,
        camera: &Camera,
        position: cgmath::Vector3<f32>,
        target: cgmath::Vector3<f32>,
    ) {
        let proj_view_matrix = super::util::generate_view_matrix(
            camera,
            position,
            target,
            graphics_context.window_size.width as f32 / graphics_context.window_size.height as f32,
        );

        graphics_context.queue.write_buffer(
            &self.global_uniform_buf,
            0,
            bytemuck::bytes_of(&GlobalUniforms {
                projection_view_matrix: proj_view_matrix.into(),
                eye_position: [position.x, position.y, position.z, 0.0],
            }),
        );
    }

    fn create_depth_view(
        device: &wgpu::Device,
        size: winit::dpi::PhysicalSize<u32>,
    ) -> wgpu::TextureView {
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: size.width,
                height: size.height,
                depth: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: super::DEPTH_FORMAT,
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
        });

        return depth_texture.create_view(&Default::default());
    }

    fn compile_pipeline(
        device: &wgpu::Device,
        pipeline_layout: &wgpu::PipelineLayout,
        vs_module: &wgpu::ShaderModule,
        fs_module: &wgpu::ShaderModule,
    ) -> wgpu::RenderPipeline {
        return device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Option::from(pipeline_layout),
            vertex: wgpu::VertexState {
                module: vs_module,
                entry_point: "main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<super::data::Vertex>() as u64,
                    step_mode: wgpu::InputStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![
                        0 => Float3,
                        1 => Float3,
                        2 => Float2
                    ],
                }],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: super::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: Default::default(),
                bias: Default::default(),
                clamp_depth: false,
            }),
            fragment: Some(wgpu::FragmentState {
                module: fs_module,
                entry_point: "main",
                targets: &[wgpu::ColorTargetState {
                    format: super::COLOR_FORMAT,
                    alpha_blend: wgpu::BlendState::REPLACE, // For now
                    color_blend: wgpu::BlendState::REPLACE,
                    write_mask: wgpu::ColorWrite::ALL,
                }],
            }),
            multisample: wgpu::MultisampleState::default(),
        });
    }
}
