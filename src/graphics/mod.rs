use zerocopy::{AsBytes, FromBytes};

pub const COLOR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;
pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

pub const MAX_NR_OF_POINT_LIGHTS: usize = 10;

pub mod data;

use cgmath::Vector3;
use imgui::FontSource;
use imgui_wgpu::RendererConfig;
use wgpu::util::DeviceExt;
use wgpu::{
    Device, PipelineLayout, RenderPipeline, ShaderModule, StencilStateDescriptor, Surface,
    SwapChain, SwapChainDescriptor,
};
use winit::dpi::PhysicalSize;
use winit::event::Event;
use winit::window::Window;

// How dirty of me
use crate::graphics::data::*;

extern crate imgui_winit_support;

pub fn sc_desc_from_size(size: &PhysicalSize<u32>) -> wgpu::SwapChainDescriptor {
    wgpu::SwapChainDescriptor {
        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        format: COLOR_FORMAT,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Mailbox,
    }
}

/// Contains everything necessary to render GUI elements.
///
/// # notes
/// + Currently focuses on Dear-ImGui using imgui-wgpu-rs.
/// + May want to expand this in the future to more GUI libraries.
///
///
pub struct GuiContext {
    pub imgui: imgui::Context,
    pub imgui_platform: imgui_winit_support::WinitPlatform,
    pub imgui_renderer: imgui_wgpu::Renderer,
}

impl GuiContext {
    pub fn new(window: &winit::window::Window, context: &Context) -> Self {
        let mut imgui = imgui::Context::create();
        let mut imgui_platform = imgui_winit_support::WinitPlatform::init(&mut imgui);

        imgui_platform.attach_window(
            imgui.io_mut(),
            &window,
            imgui_winit_support::HiDpiMode::Default,
        );

        imgui.set_ini_filename(None);

        let font_size = (13.0 * window.scale_factor()) as f32;
        imgui.io_mut().font_global_scale = (1.0 / window.scale_factor()) as f32;

        imgui.fonts().add_font(&[FontSource::DefaultFontData {
            config: Some(imgui::FontConfig {
                oversample_h: 1,
                pixel_snap_h: true,
                size_pixels: font_size,
                ..Default::default()
            }),
        }]);

        let imgui_renderer = imgui_wgpu::Renderer::new(
            &mut imgui,
            &context.device,
            &context.queue,
            RendererConfig {
                texture_format: crate::graphics::COLOR_FORMAT,
                ..Default::default()
            },
        );

        return Self {
            imgui,
            imgui_platform,
            imgui_renderer,
        };
    }

    pub fn handle_event(
        &mut self,
        window: &mut winit::window::Window,
        event: &winit::event::Event<()>,
    ) {
        self.imgui_platform
            .handle_event(self.imgui.io_mut(), window, event);
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
}

const FRAG_SRC: &str = include_str!("../../shaders/forward.frag");
const VERT_SRC: &str = include_str!("../../shaders/forward.vert");

impl Context {
    pub async fn new(window: &Window, instance: &wgpu::Instance) -> Self {
        let size = window.inner_size();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
                compatible_surface: None,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    shader_validation: true,
                },
                None,
            )
            .await
            .unwrap();

        let depth_view = Context::create_depth_view(&device, size);

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::UniformBuffer {
                        dynamic: false,
                        min_binding_size: None,
                    }, // TODO: ?
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::UniformBuffer {
                        dynamic: false,
                        min_binding_size: None,
                    }, // TODO: ?
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
                    ty: wgpu::BindingType::UniformBuffer {
                        dynamic: false,
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
                    resource: wgpu::BindingResource::Buffer(uniform_buf.slice(..)),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(lights_buf.slice(..)),
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

        let vs_module =
            device.create_shader_module(wgpu::util::make_spirv(&vs_spirv.as_binary_u8()));
        let fs_module =
            device.create_shader_module(wgpu::util::make_spirv(&fs_spirv.as_binary_u8()));

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
        };

        return context;
    }

    // Note(Jökull): A step in the right direction, but a bit heavy-handed
    pub fn model_bind_group_from_uniform_data(
        &self,
        local_uniforms: LocalUniforms,
    ) -> (wgpu::Buffer, wgpu::BindGroup) {
        let uniforms_size = std::mem::size_of::<LocalUniforms>() as u64;

        let uniform_buf = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Local Uniforms"),
                contents: local_uniforms.as_bytes(),
                usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            });

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.local_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(uniform_buf.slice(..)),
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
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &fs_module,
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back,
                clamp_depth: false,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[wgpu::ColorStateDescriptor {
                format: COLOR_FORMAT,
                alpha_blend: wgpu::BlendDescriptor {
                    src_factor: wgpu::BlendFactor::SrcColor,
                    dst_factor: wgpu::BlendFactor::DstColor,
                    operation: wgpu::BlendOperation::Add,
                },
                color_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: Some(wgpu::DepthStencilStateDescriptor {
                format: DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilStateDescriptor::default(),
            }),
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[wgpu::VertexBufferDescriptor {
                    stride: std::mem::size_of::<Vertex>() as u64,
                    step_mode: wgpu::InputStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![
                        0 => Float3,
                        1 => Float3,
                        2 => Float2
                    ],
                }],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });
    }

    pub fn resize(
        &mut self,
        size: PhysicalSize<u32>,
        sc_desc: &SwapChainDescriptor,
        surface: &Surface,
    ) -> SwapChain {
        self.depth_view = Context::create_depth_view(&self.device, size);

        return self.device.create_swap_chain(surface, sc_desc);
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
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
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

pub fn correction_matrix() -> cgmath::Matrix4<f32> {
    cgmath::Matrix4::new(
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.5, 1.0,
    )
}
