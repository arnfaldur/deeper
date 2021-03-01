use cgmath::{Deg, Vector3};
use wgpu::{
    ColorTargetState, DepthStencilState, Device, PipelineLayout, RenderPipeline, ShaderModule,
    Surface, SwapChain, SwapChainDescriptor,
};
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;
use winit::window::Window;
use zerocopy::AsBytes;

use crate::components::{Camera, Model3D, StaticModel};
// How dirty of me
use crate::graphics::data::*;
use crate::loader::AssetManager;

pub const COLOR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;
pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

pub const MAX_NR_OF_POINT_LIGHTS: usize = 10;

pub mod data;
pub mod gui;


pub fn sc_desc_from_size(size: &PhysicalSize<u32>) -> wgpu::SwapChainDescriptor {
    wgpu::SwapChainDescriptor {
        usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
        format: COLOR_FORMAT,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Mailbox,
    }
}

struct ModelQueue {
    local_uniforms: Vec<LocalUniforms>,
    model_desc: Vec<Model3D>,
    static_models: Vec<StaticModel>,
}

impl ModelQueue {
    fn new() -> Self {
        Self {
            local_uniforms: vec![],
            model_desc: vec![],
            static_models: vec![],
        }
    }

    fn clear(&mut self) {
        self.local_uniforms.clear();
        self.model_desc.clear();
        self.static_models.clear();
    }
}

/** The graphics context.
    Currently just a grab-bag of all the state and functionality
    needed to power all graphics. Needs simplification.
*/
pub struct Context {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,

    pub uniform_buf: wgpu::Buffer,
    pub lights_buf: wgpu::Buffer,

    pub local_bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group_layout: wgpu::BindGroupLayout,

    pub bind_group: wgpu::BindGroup,

    pub pipeline_layout: wgpu::PipelineLayout,
    pub pipeline: wgpu::RenderPipeline,

    pub depth_view: wgpu::TextureView,

    pub surface: wgpu::Surface,
    pub swap_chain: wgpu::SwapChain,
    pub sc_desc: wgpu::SwapChainDescriptor,

    model_queue: ModelQueue,
}

const FRAG_SRC: &str = include_str!("../../shaders/forward.frag");
const VERT_SRC: &str = include_str!("../../shaders/forward.vert");

impl Context {
    pub async fn new(window: &Window) -> Self {
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);

        let size = window.inner_size();

        let surface = unsafe { instance.create_surface(window) };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .unwrap();

        let sc_desc = sc_desc_from_size(&size);
        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        let depth_view = Context::create_depth_view(&device, size);

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let local_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let global_uniforms: GlobalUniforms = Default::default();

        let uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Global Shader Uniforms"),
            contents: global_uniforms.as_bytes(),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let lights: Lights = Default::default();

        let lights_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Lights"),
            contents: lights.as_bytes(),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &uniform_buf,
                        offset: 0,
                        size: None,
                    },
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &lights_buf,
                        offset: 0,
                        size: None,
                    },
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout, &local_bind_group_layout],
            push_constant_ranges: &[],
        });

        let mut shader_compiler = shaderc::Compiler::new().unwrap();
        let vs_spirv = shader_compiler
            .compile_into_spirv(
                VERT_SRC,
                shaderc::ShaderKind::Vertex,
                "forward.vert",
                "main",
                None,
            )
            .unwrap();
        let fs_spirv = shader_compiler
            .compile_into_spirv(
                FRAG_SRC,
                shaderc::ShaderKind::Fragment,
                "forward.frag",
                "main",
                None,
            )
            .unwrap();

        let vs_module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Vertex Shader"),
            source: wgpu::util::make_spirv(&vs_spirv.as_binary_u8()),
            flags: wgpu::ShaderFlags::default(),
        });
        let fs_module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Fragment Shader"),
            source: wgpu::util::make_spirv(&fs_spirv.as_binary_u8()),
            flags: wgpu::ShaderFlags::default(),
        });

        let pipeline = Context::compile_pipeline(&device, &pipeline_layout, vs_module, fs_module);

        let context = Context {
            device,
            queue,
            uniform_buf,
            lights_buf,
            local_bind_group_layout,
            bind_group_layout,
            bind_group,
            pipeline_layout,
            pipeline,
            depth_view,
            surface,
            swap_chain,
            sc_desc,
            model_queue: ModelQueue::new(),
        };

        return context;
    }

    pub fn draw_static_model(&mut self, model: StaticModel) {
        self.model_queue.static_models.push(model);
    }

    pub fn draw_model(
        &mut self,
        model: Model3D,
        transform: cgmath::Matrix4<f32>,
        // position: Vector3<f32>,
        // rotation: Option<Deg<f32>>,
    ) {
        use cgmath::Matrix4;

        let mut matrix = Matrix4::from_scale(model.scale);
        matrix = transform * matrix;
        //
        // if let Some(rot) = rotation {
        //     matrix = Matrix4::from_angle_z(rot) * matrix;
        // }
        //
        // matrix = Matrix4::from_translation(position) * matrix;

        self.model_queue.local_uniforms.push(LocalUniforms {
            model_matrix: matrix.into(),
            material: model.material,
        });

        self.model_queue.model_desc.push(model);
    }

    fn get_encoder(&mut self) -> wgpu::CommandEncoder {
        self.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None })
    }

    pub fn set_3d_camera(&mut self, camera: &Camera, position: Vector3<f32>, target: Vector3<f32>) {
        let mut encoder = self.get_encoder();

        let proj_view_matrix = generate_view_matrix(
            camera,
            position,
            target,
            self.sc_desc.width as f32 / self.sc_desc.height as f32,
        );

        let global_uniforms = GlobalUniforms {
            projection_view_matrix: proj_view_matrix.into(),
            eye_position: [position.x, position.y, position.z, 0.0],
        };

        let new_uniform_buf = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: global_uniforms.as_bytes(),
                usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_SRC,
            });

        encoder.copy_buffer_to_buffer(
            &new_uniform_buf,
            0,
            &self.uniform_buf,
            0,
            std::mem::size_of::<GlobalUniforms>() as u64,
        );

        self.queue.submit(std::iter::once(encoder.finish()));
    }

    pub fn render(
        &mut self,
        ass_man: &AssetManager,
        gui_context: &mut gui::GuiContext,
        window: &winit::window::Window,
    ) {
        let mut encoder = self.get_encoder();
        let current_frame = self.swap_chain.get_current_frame().unwrap();

        // Copy local uniforms
        {
            let temp_buf = self
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: self.model_queue.local_uniforms.as_bytes(),
                    usage: wgpu::BufferUsage::COPY_SRC,
                });

            for (i, model) in self.model_queue.model_desc.iter().enumerate() {
                encoder.copy_buffer_to_buffer(
                    &temp_buf,
                    (i * std::mem::size_of::<LocalUniforms>()) as u64,
                    &model.uniform_buffer,
                    0,
                    std::mem::size_of::<LocalUniforms>() as u64,
                );
            }
        }

        // Do big boi render pass
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &current_frame.output.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &self.depth_view,
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

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.bind_group, &[]);

            // render static meshes
            for model in &self.model_queue.static_models {
                render_pass.set_bind_group(1, &model.bind_group, &[]);
                for mesh in &ass_man.models[model.idx].meshes {
                    render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                    render_pass.draw(0..mesh.num_vertices as u32, 0..1)
                }
            }
            // render dynamic meshes
            for model_desc in &self.model_queue.model_desc {
                render_pass.set_bind_group(1, &model_desc.bind_group, &[]);
                for mesh in &ass_man.models[model_desc.idx].meshes {
                    render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                    render_pass.draw(0..mesh.num_vertices as u32, 0..1)
                }
            }
        }

        unsafe {
            if let Some(ui) = gui::current_ui() {
                gui_context.imgui_platform.prepare_render(&ui, &window);
            }
        }

        {
            let draw_data = unsafe {
                crate::graphics::gui::CURRENT_UI = None;
                imgui::sys::igRender();
                &*(imgui::sys::igGetDrawData() as *mut imgui::DrawData)
            };

            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
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
                .render(
                    draw_data,
                    &self.queue,
                    &self.device,
                    &mut rpass,
                )
                .expect("Rendering failed");
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        self.model_queue.clear();
    }

    // Note(Jökull): A step in the right direction, but a bit heavy-handed
    pub fn model_bind_group_from_uniform_data(
        &self,
        local_uniforms: LocalUniforms,
    ) -> (wgpu::Buffer, wgpu::BindGroup) {
        let _uniforms_size = std::mem::size_of::<LocalUniforms>() as u64;

        let uniform_buf = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: local_uniforms.as_bytes(),
                usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            });

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.local_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer {
                    buffer: &uniform_buf,
                    offset: 0,
                    size: None,
                },
            }],
        });

        (uniform_buf, bind_group)
    }

    pub fn recompile_pipeline(&mut self, vs_module: ShaderModule, fs_module: ShaderModule) {
        self.pipeline =
            Context::compile_pipeline(&self.device, &self.pipeline_layout, vs_module, fs_module);
    }

    fn compile_pipeline(
        device: &Device,
        pipeline_layout: &PipelineLayout,
        vs_module: ShaderModule,
        fs_module: ShaderModule,
    ) -> RenderPipeline {
        return device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Option::from(pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vs_module,
                entry_point: "main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as u64,
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
                format: DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: Default::default(),
                bias: Default::default(),
                clamp_depth: false,
            }),
            fragment: Some(wgpu::FragmentState {
                module: &fs_module,
                entry_point: "main",
                targets: &[wgpu::ColorTargetState {
                    format: COLOR_FORMAT,
                    alpha_blend: wgpu::BlendState::REPLACE, // For now
                    color_blend: wgpu::BlendState::REPLACE,
                    write_mask: wgpu::ColorWrite::ALL,
                }],
            }),
            multisample: wgpu::MultisampleState::default(),
        });
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.sc_desc = sc_desc_from_size(&size);
        self.depth_view = Context::create_depth_view(&self.device, size);
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
    }

    fn create_depth_view(device: &wgpu::Device, size: PhysicalSize<u32>) -> wgpu::TextureView {
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: size.width as u32,
                height: size.height as u32,
                depth: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: DEPTH_FORMAT,
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
        });

        return depth_texture.create_view(&Default::default());
    }
}

pub fn create_texels(size: usize) -> Vec<u8> {
    use std::iter;

    (0..size * size)
        .flat_map(|id| {
            // get high five for recognizing this ;)
            let cx = 3.0 * (id % size) as f32 / (size - 1) as f32 - 2.0;
            let cy = 2.0 * (id / size) as f32 / (size - 1) as f32 - 1.0;
            let (mut x, mut y, mut count) = (cx, cy, 0);
            while count < 0xFF && x * x + y * y < 4.0 {
                let old_x = x;
                x = x * x - y * y + cx;
                y = 2.0 * old_x * y + cy;
                count += 1;
            }
            iter::once(0xFF - (count * 5) as u8)
                .chain(iter::once(0xFF - (count * 15) as u8))
                .chain(iter::once(0xFF - (count * 50) as u8))
                .chain(iter::once(1))
        })
        .collect()
}

pub fn to_pos3<T>(vec: cgmath::Vector3<T>) -> cgmath::Point3<T> {
    cgmath::Point3::new(vec.x, vec.y, vec.z)
}

fn pos3(x: f32, y: f32, z: f32) -> cgmath::Point3<f32> { cgmath::Point3::new(x, y, z) }

pub fn to_vec2<T>(vec3: cgmath::Vector3<T>) -> cgmath::Vector2<T> {
    cgmath::Vector2::new(vec3.x, vec3.y)
}

pub fn generate_matrix(aspect_ratio: f32, t: f32) -> cgmath::Matrix4<f32> {
    let mx_projection = cgmath::perspective(cgmath::Deg(45f32), aspect_ratio, 1.0, 10.0);
    let mx_view = cgmath::Matrix4::look_at_rh(
        pos3(5. * t.cos(), 5.0 * t.sin(), 3.),
        pos3(0., 0., 0.),
        cgmath::Vector3::unit_z(),
    );
    let mx_correction = correction_matrix();
    return mx_correction * mx_projection * mx_view;
}

// Function by Vallentin
// https://vallentin.dev/2019/08/12/screen-to-world-cgmath

pub fn project_screen_to_world(
    screen: cgmath::Vector3<f32>,
    view_projection: cgmath::Matrix4<f32>,
    viewport: cgmath::Vector4<i32>,
) -> Option<cgmath::Vector3<f32>> {
    use cgmath::SquareMatrix;
    if let Some(inv_view_projection) = view_projection.invert() {
        let world = cgmath::Vector4::new(
            (screen.x - (viewport.x as f32)) / (viewport.z as f32) * 2.0 - 1.0,
            // Screen Origin is Top Left    (Mouse Origin is Top Left)
            // (screen.y - (viewport.y as f32)) / (viewport.w as f32) * 2.0 - 1.0,
            // Screen Origin is Bottom Left (Mouse Origin is Top Left)
            (1.0 - (screen.y - (viewport.y as f32)) / (viewport.w as f32)) * 2.0 - 1.0,
            screen.z * 2.0 - 1.0,
            1.0,
        );
        let world = inv_view_projection * world;

        if world.w != 0.0 {
            Some(world.truncate() * (1.0 / world.w))
        } else {
            None
        }
    } else {
        None
    }
}

// Function by Vallentin
// https://vallentin.dev/2019/08/12/screen-to-world-cgmath

pub fn project_world_to_screen(
    world: cgmath::Vector3<f32>,
    view_projection: cgmath::Matrix4<f32>,
    viewport: cgmath::Vector4<i32>,
) -> Option<cgmath::Vector3<f32>> {
    let screen = view_projection * world.extend(1.0);

    if screen.w != 0.0 {
        let mut screen = screen.truncate() * (1.0 / screen.w);

        screen.x = (screen.x + 1.0) * 0.5 * (viewport.z as f32) + (viewport.x as f32);
        // Screen Origin is Top Left    (Mouse Origin is Top Left)
        // screen.y = (screen.y + 1.0) * 0.5 * (viewport.w as f32) + (viewport.y as f32);
        // Screen Origin is Bottom Left (Mouse Origin is Top Left)
        screen.y = (1.0 - screen.y) * 0.5 * (viewport.w as f32) + (viewport.y as f32);

        // This is only correct when glDepthRangef(0.0f, 1.0f)
        screen.z = (screen.z + 1.0) * 0.5;

        Some(screen)
    } else {
        None
    }
}

fn generate_view_matrix(
    cam: &Camera,
    cam_pos: cgmath::Vector3<f32>,
    cam_target: cgmath::Vector3<f32>,
    aspect_ratio: f32,
) -> cgmath::Matrix4<f32> {
    let mx_correction = correction_matrix();

    let mx_view = cgmath::Matrix4::look_at_rh(
        to_pos3(cam_pos),
        to_pos3(cam_target),
        cgmath::Vector3::unit_z(),
    );

    let mx_perspective = cgmath::perspective(cgmath::Deg(cam.fov), aspect_ratio, 1.0, 1000.0);

    mx_correction * mx_perspective * mx_view
}

#[rustfmt::skip]
pub fn correction_matrix() -> cgmath::Matrix4<f32> {
    cgmath::Matrix4::new(
        1.0, 0.0, 0.0, 0.0, 
        0.0, 1.0, 0.0, 0.0, 
        0.0, 0.0, 0.5, 0.0, 
        0.0, 0.0, 0.5, 1.0,
    )
}
