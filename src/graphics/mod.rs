use zerocopy::{AsBytes, FromBytes};
use std::sync::Arc;
use lazy_static::lazy_static;

pub const COLOR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;
pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

pub const MAX_NR_OF_POINT_LIGHTS: usize = 10;

#[repr(C)]
#[derive(Clone, Copy, AsBytes, FromBytes)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coord: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, AsBytes, FromBytes, Default)]
pub struct GlobalUniforms {
    pub projection_view_matrix: [[f32; 4]; 4],
    pub eye_position: [f32; 4],
    pub time: f32,
}

#[repr(C)]
#[derive(Clone, Copy, AsBytes, FromBytes, Default)]
pub struct Material {
    pub albedo : [f32; 4],
    pub metallic : f32,
    //_pad1 : [f32; 3],
    pub roughness : f32,
    //_pad2 : [f32; 3],
}

impl Material {
    pub fn default() -> Self {
        let mut mat : Self = Default::default();
        mat.albedo = [1.0, 1.0, 1.0, 1.0];
        mat.metallic = 0.1;
        mat.roughness = 0.15;
        return mat;
    }

    pub fn glossy(color : Vector3<f32>) -> Self {
        let mut mat : Self = Self::default();
        mat.albedo = [color.x, color.y, color.z, 1.0];
        mat.roughness = 0.2;
        mat.metallic = 0.2;
        return mat;
    }

    pub fn darkest_stone() -> Self {
        let mut mat : Self = Self::default();
        mat.albedo = [0.05, 0.05, 0.05, 1.0];
        mat.metallic = 0.0;
        mat.roughness = 0.5;
        return mat;
    }

    pub fn dark_stone() -> Self {
        let mut mat : Self = Self::default();
        mat.albedo = [0.07, 0.07 , 0.07, 1.0];
        mat.metallic = 0.0;
        mat.roughness = 0.7;
        return mat;
    }

    pub fn bright_stone() -> Self {
        let mut mat : Self = Default::default();
        mat.albedo = [0.2, 0.2 , 0.2, 1.0];
        mat.metallic = 0.01;
        mat.roughness = 0.6;
        return mat;
    }
}

#[repr(C)]
#[derive(Clone, Copy, AsBytes, FromBytes)]
pub struct LocalUniforms {
    pub model_matrix: [[f32; 4]; 4],
    pub material: Material,
}

impl LocalUniforms {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            model_matrix: cgmath::Matrix4::identity().into(),
            material: Material::default(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, AsBytes, FromBytes, Default)]
pub struct DirectionalLight {
    pub direction : [f32; 4],
    pub ambient   : [f32; 4],
    pub color     : [f32; 4],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, AsBytes, FromBytes, Default)]
pub struct PointLight {
    pub radius : f32,
    pub pad : [f32; 3],
    pub position : [f32; 4],
    pub color    : [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, AsBytes, FromBytes, Default)]
pub struct Lights {
    pub directional_light : DirectionalLight,
    pub point_lights      : [PointLight; MAX_NR_OF_POINT_LIGHTS],
}

pub struct Mesh {
    pub num_vertices   : usize,
    pub vertex_buffer  : wgpu::Buffer,
    pub offset         : [f32; 3],
}

pub struct Model {
    pub meshes: Vec<Mesh>,
}

use std::path::Path;
use wavefront_obj::obj;
use std::fs::File;
use std::io::Read;
use wgpu::{BufferDescriptor, TextureView, ShaderModule, RenderPipeline, PipelineLayout, Device};
use cgmath::Vector3;
use winit::window::Window;
use winit::dpi::PhysicalSize;

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
    pub sc_desc: wgpu::SwapChainDescriptor,
    pub swap_chain: wgpu::SwapChain,
}

const FRAG_SRC: &str = include_str!("../../shaders/forward.frag");
const VERT_SRC: &str = include_str!("../../shaders/forward.vert");

impl Context {
    pub async fn new(window: &Window) -> Self {
        let surface = wgpu::Surface::create(window);

        let size = window.inner_size();

        let adapter = wgpu::Adapter::request(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
            },
            wgpu::BackendBit::PRIMARY,
        ).await.unwrap();

        let (device, mut queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                extensions: wgpu::Extensions {
                    anisotropic_filtering: false
                },
                limits: Default::default(),
            }
        ).await;

        let mut sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: COLOR_FORMAT,
            width: size.width as u32,
            height: size.height as u32,
            present_mode: wgpu::PresentMode::Mailbox,
        };

        let mut swap_chain = device.create_swap_chain(&surface, &sc_desc);

        let depth_view = Context::create_depth_view(&device, size);

        let bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                bindings: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::UniformBuffer { dynamic: false } // TODO: ?
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::UniformBuffer { dynamic: false } // TODO: ?
                    },
                ],
            }
        );

        let local_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                bindings: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::UniformBuffer { dynamic: false }
                    }
                ]
            }
        );

        let global_uniforms : GlobalUniforms = Default::default();

        let uniform_buf = device.create_buffer_with_data(
            global_uniforms.as_bytes(),
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        );

        let lights : Lights = Default::default();

        let lights_buf = device.create_buffer_with_data(
            lights.as_bytes(),
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        );

        let mut bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &uniform_buf,
                        range: 0..std::mem::size_of::<GlobalUniforms>() as u64,
                    },
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &lights_buf,
                        range: 0..std::mem::size_of::<Lights>() as u64,
                    }
                }
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[&bind_group_layout, &local_bind_group_layout]
            }
        );

        use glsl_to_spirv::ShaderType;
        let vs = glsl_to_spirv::compile(VERT_SRC, ShaderType::Vertex).unwrap();
        let vs_module = device.create_shader_module(&wgpu::read_spirv(vs).unwrap());

        let fs = glsl_to_spirv::compile(FRAG_SRC, ShaderType::Fragment).unwrap();
        let fs_module = device.create_shader_module(&wgpu::read_spirv(fs).unwrap());

        let pipeline = Context::compile_pipeline(&device, &pipeline_layout, vs_module, fs_module);

        let mut context = Context {
            device,
            queue,
            surface,
            sc_desc,
            swap_chain,
            uniform_buf,
            lights_buf,
            local_bind_group_layout,
            bind_group_layout,
            bind_group,
            pipeline_layout,
            pipeline,
            depth_view
        };

        return context;
    }

    pub fn recompile_pipeline(&mut self, vs_module: ShaderModule, fs_module: ShaderModule) {
        self.pipeline = Context::compile_pipeline(&self.device, &self.pipeline_layout, vs_module, fs_module);
    }

    fn compile_pipeline(device: &Device, pipeline_layout: &PipelineLayout, vs_module : ShaderModule, fs_module : ShaderModule) -> RenderPipeline{
        return device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: pipeline_layout,
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
                cull_mode: wgpu::CullMode::None,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0
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
                stencil_front: wgpu::StencilStateFaceDescriptor::IGNORE,
                stencil_back : wgpu::StencilStateFaceDescriptor::IGNORE,
                stencil_read_mask: !0,
                stencil_write_mask: !0,
            }),
            index_format: wgpu::IndexFormat::Uint16,
            vertex_buffers: &[wgpu::VertexBufferDescriptor{
                stride: std::mem::size_of::<Vertex>() as u64,
                step_mode: wgpu::InputStepMode::Vertex,
                attributes: &wgpu::vertex_attr_array![
                    0 => Float3,
                    1 => Float3,
                    2 => Float2
                ]
            }],
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false
        });
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.sc_desc.width  = size.width;
        self.sc_desc.height = size.height;

        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);

        self.depth_view = Context::create_depth_view(&self.device, size);
    }

    fn create_depth_view(device: &wgpu::Device, size: PhysicalSize<u32>) -> wgpu::TextureView {
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: size.width as u32,
                height: size.height as u32,
                depth: 1,
            },
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: DEPTH_FORMAT,
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        });

        return depth_texture.create_default_view();
    }

    pub fn load_model_from_obj(&self, path: &Path) -> Model {
        let mut vertex_lists = vertex_lists_from_obj(path).unwrap();
        return self.load_model_from_vertex_lists(&vertex_lists);
    }

    pub fn load_model_from_vertex_lists(&self, vertex_lists: &Vec<Vec<Vertex>>) -> Model {
        let mut meshes = vec!();

        for vertices in vertex_lists {

            let vertex_buf = self.device.create_buffer_with_data(
                vertices.as_bytes(),
                wgpu::BufferUsage::VERTEX,
            );

            meshes.push(
                Mesh {
                    num_vertices: vertices.len(),
                    vertex_buffer: vertex_buf,
                    offset: [0.0, 0.0, 0.0],
                }
            );
        }

        Model { meshes }
    }

}

pub fn vertex_lists_from_obj(path: &Path) -> Result<Vec<Vec<Vertex>>, String> {
    let mut f;

    if let Ok(file) = File::open(path) {
        f = file;
    } else {
        return Err(format!("[graphics] : File {} could not be opened.", path.display()));
    };

    let mut buf = String::new();
    f.read_to_string(&mut buf);

    let obj_set = obj::parse(buf)
        .expect("Failed to parse obj file");

    let mut vertex_lists = vec!();

    for obj in &obj_set.objects {

        let mut vertices = vec!();

        for g in &obj.geometry {
            let mut indices = vec!();

            g.shapes.iter().for_each(|shape| {
                if let obj::Primitive::Triangle(v1, v2, v3) = shape.primitive {
                    indices.push(v1);
                    indices.push(v2);
                    indices.push(v3);
                }
            });

            for idx in &indices {
                let pos = obj.vertices[idx.0];
                let normal = match idx.2 {
                    Some(i) => obj.normals[i],
                    _ => obj::Normal{ x: 0.0, y: 0.0, z: 0.0}
                };
                let tc = match idx.1 {
                    Some(i) => obj.tex_vertices[i],
                    _ => obj::TVertex{ u: 0.0, v: 0.0, w: 0.0}
                };
                let v = Vertex {
                    pos: [pos.x as f32, pos.y as f32, pos.z as f32],
                    normal: [normal.x as f32, normal.y as f32, normal.z as f32],
                    tex_coord: [tc.u as f32, tc.v as f32]
                };
                vertices.push(v);
            }
        }
        vertex_lists.push(vertices);
    }
    Ok(vertex_lists)

}

pub fn create_texels(size: usize) -> Vec<u8> {
    use std::iter;

    (0 .. size * size)
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

fn pos3(x: f32, y: f32, z: f32) -> cgmath::Point3<f32> {
    cgmath::Point3::new(x, y, z)
}

pub fn to_vec2<T>(vec3: cgmath::Vector3<T>) -> cgmath::Vector2<T> {
    cgmath::Vector2::new(vec3.x, vec3.y)
}

pub fn generate_matrix(aspect_ratio: f32, t : f32) -> cgmath::Matrix4<f32> {
    let mx_projection = cgmath::perspective(cgmath::Deg(45f32), aspect_ratio, 1.0, 10.0);
    let mx_view = cgmath::Matrix4::look_at(
        pos3(5.0  * t.cos(), 5.0 * t.sin(), 3.0),
        pos3(0f32, 0.0, 0.0),
        cgmath::Vector3::unit_z(),
    );
    let mx_correction = cgmath::Matrix4::new(
        1.0, 0.0, 0.0, 0.0,
        0.0, -1.0, 0.0, 0.0,
        0.0, 0.0, 0.5, 0.0,
        0.0, 0.0, 0.5, 1.0,
    );
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
            (1.0 - (screen.y - (viewport.y as f32)) / (viewport.w as f32)) * 2.0 - 1.0, screen.z * 2.0 - 1.0,
            1.0);
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
        1.0, 0.0, 0.0, 0.0,
        0.0, -1.0, 0.0, 0.0,
        0.0, 0.0, 0.5, 0.0,
        0.0, 0.0, 0.5, 1.0,
    )
}
