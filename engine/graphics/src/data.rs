#![allow(unused)]
use bytemuck::{Pod, Zeroable};
use cgmath::{Matrix4, Vector3, Vector4};
use image::{EncodableLayout, GenericImageView};

use crate::MAX_NR_OF_POINT_LIGHTS;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coord: [f32; 2],
}

impl Vertex {
    pub fn transformed(&self, model_matrix: [[f32; 4]; 4]) -> Self {
        Self {
            pos: {
                (Matrix4::from(model_matrix) * Vector3::from(self.pos).extend(1.0))
                    .truncate()
                    .into()
            },
            normal: {
                (Matrix4::from(model_matrix) * Vector3::from(self.normal).extend(0.0))
                    .truncate()
                    .into()
            },
            tex_coord: self.tex_coord,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Default, Pod, Zeroable)]
pub struct GlobalUniforms {
    pub projection_view_matrix: [[f32; 4]; 4],
    pub eye_position: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct Material {
    pub albedo: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            albedo: [1.0, 1.0, 1.0, 1.0],
            metallic: 0.1,
            roughness: 0.15,
        }
    }
}

impl Material {
    pub fn color(color: Vector4<f32>) -> Self {
        Self {
            albedo: color.into(),
            metallic: 0.0,
            roughness: 0.0,
        }
    }

    pub fn glossy(color: Vector3<f32>) -> Self {
        Self {
            albedo: [color.x, color.y, color.z, 1.0],
            metallic: 0.2,
            roughness: 0.2,
        }
    }

    pub fn darkest_stone() -> Self {
        Self {
            albedo: [0.05, 0.05, 0.05, 1.0],
            metallic: 0.0,
            roughness: 0.5,
        }
    }

    pub fn dark_stone() -> Self {
        Self {
            albedo: [0.07, 0.07, 0.07, 1.0],
            metallic: 0.0,
            roughness: 0.7,
        }
    }
}

// TODO: make it so we don't need to manually manage this somehow
const LU_BYTES: usize = std::mem::size_of::<[[f32; 4]; 4]>() + std::mem::size_of::<Material>();
const LU_ALIGN: usize = wgpu::BIND_BUFFER_ALIGNMENT as usize - LU_BYTES;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Default)]
pub struct LocalUniforms {
    pub model_matrix: [[f32; 4]; 4],
    pub material: Material,
    _align: [f64; LU_ALIGN / 8],
}

impl LocalUniforms {
    pub fn new(model_matrix: [[f32; 4]; 4], material: Material) -> Self {
        Self {
            model_matrix,
            material,
            _align: [0.0; LU_ALIGN / 8],
        }
    }

    pub fn simple(
        translation: [f32; 3],
        scale: f32,
        z_rotation_deg: f32,
        material: Material,
    ) -> Self {
        let mut matrix = Matrix4::from_scale(scale);
        matrix = Matrix4::from_angle_z(cgmath::Deg(z_rotation_deg)) * matrix;
        matrix = Matrix4::from_translation(translation.into()) * matrix;

        Self::new(matrix.into(), material)
    }

    pub fn with_model_matrix(&self, model_matrix: [[f32; 4]; 4]) -> Self {
        Self {
            model_matrix,
            material: self.material,
            _align: self._align,
        }
    }

    pub fn with_material(&self, material: Material) -> Self {
        Self {
            model_matrix: self.model_matrix,
            material,
            _align: self._align,
        }
    }

    /// Used to gauge whether uniforms are equal besides
    /// model_matrix. Can use bevy::reflect to generalize
    pub fn similar_to(&self, other: &Self) -> bool { self.material == other.material }

    pub fn init() -> Self {
        use cgmath::SquareMatrix;
        Self {
            model_matrix: cgmath::Matrix4::identity().into(),
            material: Material::default(),
            _align: [0.0; LU_ALIGN / 8],
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Default)]
pub struct DirectionalLight {
    pub direction: [f32; 4],
    pub ambient: [f32; 4],
    pub color: [f32; 4],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable, Default)]
pub struct PointLight {
    pub radius: f32,
    pub pad: [f32; 3],
    pub position: [f32; 4],
    pub color: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Default)]
pub struct Lights {
    pub directional_light: DirectionalLight,
    pub point_lights: [PointLight; MAX_NR_OF_POINT_LIGHTS],
}

pub struct Mesh {
    pub num_vertices: usize,
    pub vertex_buffer: wgpu::Buffer,
    pub offset: [f32; 3],
}

pub type VertexLists = Vec<Vec<Vertex>>;

pub struct Model {
    pub meshes: Vec<Mesh>,
    pub vertex_lists: VertexLists,
}

pub struct Texture {
    pub texture: wgpu::Texture,
    pub texture_view: wgpu::TextureView,
    pub texture_size: wgpu::Extent3d,
    pub image: image::DynamicImage,
}

impl Texture {
    const TEXTURE_UNIFORM_BIND_GROUP_LAYOUT_DESCRIPTOR: wgpu::BindGroupLayoutDescriptor<'static> =
        wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler {
                        comparison: false,
                        filtering: true,
                    },
                    count: None,
                },
            ],
        };

    pub fn new(image: image::DynamicImage, context: &super::GraphicsContext) -> Self {
        let texture_size = wgpu::Extent3d {
            width: image.width(),
            height: image.height(),
            depth: 1,
        };
        let texture = context.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: super::COLOR_FORMAT,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
        });
        context.queue.write_texture(
            wgpu::TextureCopyView {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            image.flipv().into_bgra8().as_bytes(),
            wgpu::TextureDataLayout {
                offset: 0,
                bytes_per_row: 4 * image.width(),
                rows_per_image: image.height(),
            },
            texture_size,
        );
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: None,
            format: None,
            dimension: None,
            aspect: Default::default(),
            base_mip_level: 0,
            level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });

        Self {
            texture,
            texture_view,
            texture_size,
            image,
        }
    }
}
