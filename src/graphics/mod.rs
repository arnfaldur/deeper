use zerocopy::{AsBytes, FromBytes};
use std::sync::Arc;
use lazy_static::lazy_static;

pub const COLOR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;
pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

#[repr(C)]
#[derive(Clone, Copy, AsBytes, FromBytes)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub tex_coord: [f32; 2],
}

fn vertex(pos: [f32;3], tex_coord: [f32;2]) -> Vertex {
    Vertex { pos, tex_coord }
}

#[repr(C)]
#[derive(Clone, Copy, AsBytes, FromBytes)]
pub struct LocalUniforms {
    model_matrix: [[f32; 4]; 4],
}

impl LocalUniforms {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self { model_matrix: cgmath::Matrix4::identity().into() }
    }
}

pub struct Mesh {
    pub num_vertices   : usize,
    pub vertex_buffer  : wgpu::Buffer,
    pub uniform_buffer : wgpu::Buffer,
    pub offset         : [f32; 3],
    pub bind_group     : wgpu::BindGroup,
}

pub struct Model {
    pub meshes : Vec<Mesh>,
}

use std::path::Path;
use wavefront_obj::obj;
use std::fs::File;
use std::io::Read;
use wgpu::BufferDescriptor;

// Based on vange-rs
pub struct Context {
    pub uniform_buf: wgpu::Buffer,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    pub pipeline_layout: wgpu::PipelineLayout,
    pub pipeline: wgpu::RenderPipeline,
    pub texture: wgpu::Texture,
}

const FRAG_SRC: &str = include_str!("../../shaders/debug.frag");
const VERT_SRC: &str = include_str!("../../shaders/debug.vert");

//fn generate_local_bind_group_layout_desc() -> wgpu::BindGroupLayoutDescriptor<'static> {
//    wgpu::BindGroupLayoutDescriptor {
//        bindings: &[
//            wgpu::BindGroupLayoutEntry {
//                binding: 0,
//                visibility: wgpu::ShaderStage::all(),
//                ty: wgpu::BindingType::UniformBuffer { dynamic: false }
//            }
//        ]
//    }
//}

impl Context {
    pub fn new(device: &wgpu::Device) -> Self {

        let bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                bindings: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::all(),
                        ty: wgpu::BindingType::UniformBuffer { dynamic: false } // TODO: ?
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::SampledTexture {
                            multisampled: false,
                            // component_type: wgpu::TextureComponentType::Float,
                            dimension: wgpu::TextureViewDimension::D2,
                            component_type: wgpu::TextureComponentType::Float,
                        }
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler { comparison: false }
                    }
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

        let mx_total = generate_matrix( 1.0, 0.0);
        let mx_ref : &[f32; 16] = mx_total.as_ref();

        let uniform_buf = device.create_buffer_with_data(
            mx_ref.as_bytes(),
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        );

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare: None
        });
        let size = 256u32;

        let texture_extent = wgpu::Extent3d {
            width: size,
            height: size,
            depth: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_extent,
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
        });

        let texture_view = texture.create_default_view();

        let mut bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &uniform_buf,
                        range: 0..64,
                    },
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::Binding {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&bind_group_layout, &local_bind_group_layout]
        });

        use glsl_to_spirv::ShaderType;
        let vs = glsl_to_spirv::compile(VERT_SRC, ShaderType::Vertex).unwrap();
        let vs_module = device.create_shader_module(&wgpu::read_spirv(vs).unwrap());

        let fs = glsl_to_spirv::compile(FRAG_SRC, ShaderType::Fragment).unwrap();
        let fs_module = device.create_shader_module(&wgpu::read_spirv(fs).unwrap());

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: &pipeline_layout,
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
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[wgpu::ColorStateDescriptor {
                format: COLOR_FORMAT,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: None,
            index_format: wgpu::IndexFormat::Uint16,
            vertex_buffers: &[wgpu::VertexBufferDescriptor{
                stride: std::mem::size_of::<Vertex>() as u64,
                step_mode: wgpu::InputStepMode::Vertex,
                attributes: &[
                    wgpu::VertexAttributeDescriptor {
                        format: wgpu::VertexFormat::Float3,
                        offset: 0,
                        shader_location: 0,
                    },
                    wgpu::VertexAttributeDescriptor {
                        format: wgpu::VertexFormat::Float2,
                        offset: 3 * 4,
                        shader_location: 1,
                    }
                ]
            }],
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false
        });

        Context {uniform_buf, bind_group_layout, bind_group, pipeline_layout, pipeline, texture }
    }

    pub fn load_model_from_obj(device: &wgpu::Device, path: &str) -> Model {
        let mut f = File::open(Path::new(path))
            .expect("Failed to load obj file");
        let mut buf = String::new();
        f.read_to_string(&mut buf);

        let obj_set = obj::parse(buf)
            .expect("Failed to parse obj file");

        let bind_group_layout = 
            device.create_bind_group_layout(
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

        let mut meshes = vec!();

        let mut encoder = device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor{ todo: 0 }
        );

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
                        tex_coord: [tc.u as f32, tc.v as f32]
                    };
                    vertices.push(v);
                }
            }

            let vertex_buf = device.create_buffer_with_data(
                vertices.as_bytes(),
                wgpu::BufferUsage::VERTEX,
            );

            //let uniform_buf =
            //    device.create_buffer_mapped::<LocalUniforms>(
            //        1,
            //        wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            //    ).fill_from_slice(&[LocalUniforms::new()]);
            //

            let uniform_buf = device.create_buffer(&BufferDescriptor {
                size: 64,
                usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST
            });

            let bind_group = device.create_bind_group(
                &wgpu::BindGroupDescriptor {
                    layout: &bind_group_layout,
                    bindings: &[
                        wgpu::Binding {
                            binding: 0,
                            resource: wgpu::BindingResource::Buffer {
                                buffer: &uniform_buf,
                                range: 0..64,
                            }
                        },
                    ],
                }
            );

            meshes.push(
                Mesh {
                    num_vertices: vertices.len(),
                    vertex_buffer: vertex_buf,
                    uniform_buffer: uniform_buf,
                    offset: [0.0, 0.0, 0.0],
                    bind_group
                }
            );
        }

        Model { meshes }
    }

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

pub fn create_vertices() -> (Vec<Vertex>, Vec<u16>) {
    let vertex_data = [
        // top (0, 0, 1)
        vertex([-1.0, -1.0, 1.0], [0.0, 0.0]),
        vertex([1.0, -1.0, 1.0], [1.0, 0.0]),
        vertex([1.0, 1.0, 1.0], [1.0, 1.0]),
        vertex([-1.0, 1.0, 1.0], [0.0, 1.0]),
        // bottom (0, 0, -1)
        vertex([-1.0, 1.0, -1.0], [1.0, 0.0]),
        vertex([1.0, 1.0, -1.0], [0.0, 0.0]),
        vertex([1.0, -1.0, -1.0], [0.0, 1.0]),
        vertex([-1.0, -1.0, -1.0], [1.0, 1.0]),
        // right (1, 0, 0)
        vertex([1.0, -1.0, -1.0], [0.0, 0.0]),
        vertex([1.0, 1.0, -1.0], [1.0, 0.0]),
        vertex([1.0, 1.0, 1.0], [1.0, 1.0]),
        vertex([1.0, -1.0, 1.0], [0.0, 1.0]),
        // left (-1, 0, 0)
        vertex([-1.0, -1.0, 1.0], [1.0, 0.0]),
        vertex([-1.0, 1.0, 1.0], [0.0, 0.0]),
        vertex([-1.0, 1.0, -1.0], [0.0, 1.0]),
        vertex([-1.0, -1.0, -1.0], [1.0, 1.0]),
        // front (0.0, 1, 0)
        vertex([1.0, 1.0, -1.0], [1.0, 0.0]),
        vertex([-1.0, 1.0, -1.0], [0.0, 0.0]),
        vertex([-1.0, 1.0, 1.0], [0.0, 1.0]),
        vertex([1.0, 1.0, 1.0], [1.0, 1.0]),
        // back (0, -1, 0)
        vertex([1.0, -1.0, 1.0], [0.0, 0.0]),
        vertex([-1.0, -1.0, 1.0], [1.0, 0.0]),
        vertex([-1.0, -1.0, -1.0], [1.0, 1.0]),
        vertex([1.0, -1.0, -1.0], [0.0, 1.0]),
    ];



    let index_data: &[u16] = &[
        0, 1, 2, 2, 3, 0, // top
        4, 5, 6, 6, 7, 4, // bottom
        8, 9, 10, 10, 11, 8, // right
        12, 13, 14, 14, 15, 12, // left
        16, 17, 18, 18, 19, 16, // front
        20, 21, 22, 22, 23, 20, // back
    ];

    (vertex_data.to_vec(), index_data.to_vec())
}

fn pos3(x: f32, y: f32, z: f32) -> cgmath::Point3<f32> {
    cgmath::Point3::new(x, y, z)
}

pub(crate) fn generate_matrix(aspect_ratio: f32, t : f32) -> cgmath::Matrix4<f32> {
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
