#![allow(unused)]
use std::mem::MaybeUninit;

use cgmath::{vec2, Vector2};
use wgpu::util::DeviceExt;
use wgpu::CommandEncoderDescriptor;
use zerocopy::{AsBytes, FromBytes};

use super::data::{GlobalUniforms, LocalUniforms};
use crate::graphics::data::Material;

/*
    This module's goal is to describe an appropriate way to interpret 2D elements
    on the screen (with textures and materials) such that it can be easily chucked
    into the renderer.
*/

/**
    Describes one of 9 anchor points on a bounding rectangle from which
    a shape derives its functional center.
    
    (TL)-(TM)-(TR)
     |    |    |
    (ML)-(MM)-(MR)
     |    |    |
    (BL)-(BM)-(BR)
*/
#[rustfmt::skip]
pub enum AnchorPoint {
    TopLeft,    TopCenter,    TopRight,
    CenterLeft, Center,       CenterRight,
    BottomLeft, BottomCenter, BottomRight,
}

#[derive(Copy, Clone)]
pub enum ScreenScalar {
    Absolute(f32),
    RelativeToWidth(f32),
    RelativeToHeight(f32),
}

pub struct ScreenVector {
    value: Vector2<ScreenScalar>,
}

impl ScreenVector {
    pub fn new_absolute(x: f32, y: f32) -> Self {
        Self {
            value: vec2(ScreenScalar::Absolute(x), ScreenScalar::Absolute(y)),
        }
    }

    pub fn new_relative(x: f32, y: f32) -> Self {
        Self {
            value: vec2(
                ScreenScalar::RelativeToWidth(x),
                ScreenScalar::RelativeToHeight(y),
            ),
        }
    }

    pub fn new_relative_to_width(x: f32, y: f32) -> Self {
        Self {
            value: vec2(
                ScreenScalar::RelativeToWidth(x),
                ScreenScalar::RelativeToWidth(y),
            ),
        }
    }

    pub fn new_relative_to_height(x: f32, y: f32) -> Self {
        Self {
            value: vec2(
                ScreenScalar::RelativeToHeight(x),
                ScreenScalar::RelativeToHeight(y),
            ),
        }
    }

    pub fn as_screen_coordinates(&self, screen_size: Vector2<f32>) -> Vector2<f32> {
        self.value.map(|component| match component {
            ScreenScalar::Absolute(val) => val,
            ScreenScalar::RelativeToWidth(val) => val * screen_size.x,
            ScreenScalar::RelativeToHeight(val) => val * screen_size.y,
        })
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, AsBytes, FromBytes)]
struct CanvasVertex {
    pub position: [f32; 2],
    pub tex_coord: [f32; 2],
}

struct ImmediateElement {
    pub uniform_buf: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl ImmediateElement {
    fn new(device: &wgpu::Device) -> Self {
        let uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: LocalUniforms::new().as_bytes(),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[CanvasRenderContext::LOCAL_UNIFORM_BIND_GROUP_LAYOUT_ENTRY],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer {
                    buffer: &uniform_buf,
                    offset: 0,
                    size: None,
                },
            }],
        });

        Self {
            uniform_buf,
            bind_group,
        }
    }
}

pub enum RectangleDescriptor {
    CornerRect {
        corner1: ScreenVector,
        corner2: ScreenVector,
    },
    AnchorRect {
        anchor: AnchorPoint,
        position: ScreenVector,
        dimensions: ScreenVector,
        offset: ScreenVector,
    },
}

pub struct CanvasQueue {
    local_uniforms: Vec<super::data::LocalUniforms>,
}

impl CanvasQueue {
    pub fn new() -> Self {
        Self {
            local_uniforms: vec![],
        }
    }

    pub fn clear(&mut self) { self.local_uniforms.clear(); }

    pub fn draw_rect(
        &mut self,
        desc: RectangleDescriptor,
        color: cgmath::Vector4<f32>,
        size: winit::dpi::PhysicalSize<u32>,
    ) {
        let size = cgmath::vec2(size.width as f32, size.height as f32);

        self.local_uniforms.push({
            let (position, dimensions) = match desc {
                RectangleDescriptor::CornerRect { corner1, corner2 } => (
                    corner1.as_screen_coordinates(size),
                    corner2.as_screen_coordinates(size) - corner1.as_screen_coordinates(size),
                ),
                RectangleDescriptor::AnchorRect {
                    anchor,
                    position,
                    dimensions,
                    offset,
                } => {
                    let pos = position.as_screen_coordinates(size);
                    let off = offset.as_screen_coordinates(size);
                    let dim = dimensions.as_screen_coordinates(size);
                    match anchor {
                        AnchorPoint::TopLeft => (pos + off, dim),
                        AnchorPoint::TopCenter => (pos - vec2(dim.x / 2.0, 0.0) + off, dim),
                        AnchorPoint::TopRight => (pos - vec2(dim.x, 0.0) + off, dim),
                        AnchorPoint::CenterLeft => (pos - vec2(0.0, dim.y / 2.0) + off, dim),
                        AnchorPoint::Center => (pos - vec2(dim.x / 2.0, dim.y / 2.0) + off, dim),
                        AnchorPoint::CenterRight => (pos - vec2(0.0, dim.y / 2.0) + off, dim),
                        AnchorPoint::BottomLeft => (pos - Vector2::new(0.0, dim.y) + off, dim),
                        AnchorPoint::BottomCenter => (pos - vec2(dim.x / 2.0, dim.y) + off, dim),
                        AnchorPoint::BottomRight => (pos - Vector2::new(dim.x, dim.y) + off, dim),
                    }
                }
            };

            LocalUniforms {
                model_matrix: (cgmath::Matrix4::from_translation(position.extend(0.0))
                    * cgmath::Matrix4::from_nonuniform_scale(dimensions.x, dimensions.y, 1.0))
                .into(),
                material: Material::color(color),
            }
        });
    }
}

const CANVAS_FRAG_SRC: &str = include_str!("../../shaders/canvas.frag");
const CANVAS_VERT_SRC: &str = include_str!("../../shaders/canvas.vert");

const MAXIMUM_NUMBER_OF_QUADS: usize = 1000;

pub struct CanvasRenderContext {
    global_uniform_buf: wgpu::Buffer,

    global_bind_group: wgpu::BindGroup,

    pipeline: wgpu::RenderPipeline,

    quad_mesh: super::data::Mesh,
    immediate_elements: [ImmediateElement; MAXIMUM_NUMBER_OF_QUADS],
}

impl CanvasRenderContext {
    // Ugly workaround since the OR operation on ShaderStages is not a const-friendly operation
    pub const VERTEX_FRAGMENT_VISIBILITY: wgpu::ShaderStage = wgpu::ShaderStage::from_bits_truncate(
        wgpu::ShaderStage::VERTEX.bits() | wgpu::ShaderStage::FRAGMENT.bits(),
    );

    #[rustfmt::skip]
    const QUAD_VERTEX_LIST: [f32; (2 + 2) * 6] = [
        0.0, 0.0, 0.0, 0.0, // TL
        0.0, 1.0, 0.0, 1.0, // TR
        1.0, 0.0, 1.0, 0.0, // BL
        0.0, 1.0, 0.0, 1.0, // TR
        1.0, 1.0, 1.0, 1.0, // BR
        1.0, 0.0, 1.0, 0.0, // BL
    ];

    pub const LOCAL_UNIFORM_BIND_GROUP_LAYOUT_ENTRY: wgpu::BindGroupLayoutEntry =
        wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: Self::VERTEX_FRAGMENT_VISIBILITY,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };

    const GLOBAL_UNIFORM_BIND_GROUP_LAYOUT_ENTRY: wgpu::BindGroupLayoutEntry =
        wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: Self::VERTEX_FRAGMENT_VISIBILITY,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };

    pub fn new(device: &wgpu::Device, window_size: winit::dpi::PhysicalSize<u32>) -> Self {
        // This describes the layout of bindings to buffers in the shader program
        let global_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[Self::GLOBAL_UNIFORM_BIND_GROUP_LAYOUT_ENTRY],
            });

        let local_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[Self::LOCAL_UNIFORM_BIND_GROUP_LAYOUT_ENTRY],
            });

        let global_uniforms = GlobalUniforms {
            projection_view_matrix: super::util::generate_ortho_matrix(window_size.cast()).into(),
            eye_position: [0.0, 0.0, 1.0, 0.0],
        };

        let global_uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Global Shader Uniforms"),
            contents: global_uniforms.as_bytes(),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let global_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &global_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer {
                    buffer: &global_uniform_buf,
                    offset: 0,
                    size: None,
                },
            }],
        });

        let (vs_module, fs_module) = {
            //Todo: Move shader compilation to ass_man
            let mut shader_compiler = shaderc::Compiler::new().unwrap();

            let vs_spirv = shader_compiler
                .compile_into_spirv(
                    CANVAS_VERT_SRC,
                    shaderc::ShaderKind::Vertex,
                    "canvas.vert",
                    "main",
                    None,
                )
                .unwrap();
            let fs_spirv = shader_compiler
                .compile_into_spirv(
                    CANVAS_FRAG_SRC,
                    shaderc::ShaderKind::Fragment,
                    "canvas.frag",
                    "main",
                    None,
                )
                .unwrap();

            let vs = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
                label: Some("Canvas Vertex Shader"),
                source: wgpu::util::make_spirv(&vs_spirv.as_binary_u8()),
                flags: wgpu::ShaderFlags::default(),
            });

            let fs = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
                label: Some("Canvas Fragment Shader"),
                source: wgpu::util::make_spirv(&fs_spirv.as_binary_u8()),
                flags: wgpu::ShaderFlags::default(),
            });

            (vs, fs)
        };

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&global_bind_group_layout, &local_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = Self::compile_pipeline(&device, &pipeline_layout, vs_module, fs_module);

        // Maybe use the array crate that automates this?
        let immediate_elements: [ImmediateElement; MAXIMUM_NUMBER_OF_QUADS] = unsafe {
            let mut arr: [MaybeUninit<ImmediateElement>; MAXIMUM_NUMBER_OF_QUADS] =
                MaybeUninit::uninit().assume_init();

            for elem in &mut arr[..] {
                *elem = MaybeUninit::new(ImmediateElement::new(device));
            }

            std::mem::transmute::<_, [ImmediateElement; MAXIMUM_NUMBER_OF_QUADS]>(arr)
        };

        let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: Self::QUAD_VERTEX_LIST.as_bytes(),
            usage: wgpu::BufferUsage::VERTEX,
        });

        let quad_mesh = super::data::Mesh {
            num_vertices: 6,
            vertex_buffer: vertex_buf,
            offset: [0.0, 0.0, 0.0],
        };

        Self {
            global_uniform_buf,
            global_bind_group,
            pipeline,
            quad_mesh,
            immediate_elements,
        }
    }

    pub fn render(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        canvas_queue: &CanvasQueue,
        view: &wgpu::TextureView,
    ) {
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Canvas Render"),
        });

        // Todo: Fail more gracefully, maybe try to expand or re-use buffers?
        if canvas_queue.local_uniforms.len() > MAXIMUM_NUMBER_OF_QUADS {
            panic!("Number of Canvas Elements in frame exceeds available immediate quads.");
        }

        for (i, local_uniforms) in canvas_queue.local_uniforms.iter().enumerate() {
            queue.write_buffer(
                &self.immediate_elements.get(i).unwrap().uniform_buf,
                0,
                local_uniforms.as_bytes(),
            );
        }

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.global_bind_group, &[]);

            // render dynamic meshes
            for (_, elem) in canvas_queue
                .local_uniforms
                .iter()
                .zip(self.immediate_elements.iter())
            {
                render_pass.set_bind_group(1, &elem.bind_group, &[]);
                render_pass.set_vertex_buffer(0, self.quad_mesh.vertex_buffer.slice(..));
                render_pass.draw(0..self.quad_mesh.num_vertices as u32, 0..1)
            }
        }

        queue.submit(std::iter::once(encoder.finish()));
    }

    pub fn set_camera(&mut self, queue: &wgpu::Queue, window_size: winit::dpi::PhysicalSize<u32>) {
        let global_uniforms = GlobalUniforms {
            projection_view_matrix: super::util::generate_ortho_matrix(window_size.cast()).into(),
            eye_position: [0.0, 0.0, 1.0, 0.0],
        };

        queue.write_buffer(&self.global_uniform_buf, 0, global_uniforms.as_bytes());
    }

    fn compile_pipeline(
        device: &wgpu::Device,
        pipeline_layout: &wgpu::PipelineLayout,
        vs_module: wgpu::ShaderModule,
        fs_module: wgpu::ShaderModule,
    ) -> wgpu::RenderPipeline {
        return device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Option::from(pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vs_module,
                entry_point: "main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<CanvasVertex>() as u64,
                    step_mode: wgpu::InputStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![
                        0 => Float2,
                        2 => Float2
                    ],
                }],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            fragment: Some(wgpu::FragmentState {
                module: &fs_module,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screen_vector() {
        let screen = Vector2::new(100.0, 100.0);
        let screen_vec_rel = ScreenVector::new_relative(0.5, 0.5);
        let screen_vec_abs = ScreenVector::new_absolute(50.0, 50.0);

        assert_eq!(
            screen_vec_rel.as_screen_coordinates(screen),
            Vector2::new(50.0, 50.0)
        );
        assert_eq!(
            screen_vec_rel.as_screen_coordinates(screen),
            screen_vec_abs.as_screen_coordinates(screen)
        );
    }
}
