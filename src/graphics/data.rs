#![allow(unused)]
use bytemuck::{Pod, Zeroable};
use cgmath::{Vector3, Vector4};

use crate::graphics::MAX_NR_OF_POINT_LIGHTS;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coord: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Default, Pod, Zeroable)]
pub struct GlobalUniforms {
    pub projection_view_matrix: [[f32; 4]; 4],
    pub eye_position: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Default)]
pub struct Material {
    pub albedo: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
}

impl Material {
    pub fn default() -> Self {
        let mut mat: Self = Default::default();
        mat.albedo = [1.0, 1.0, 1.0, 1.0];
        mat.metallic = 0.1;
        mat.roughness = 0.15;
        return mat;
    }

    pub fn color(color: Vector4<f32>) -> Self {
        let mut mat: Self = Self::default();
        mat.albedo = color.into();
        mat.metallic = 0.0;
        mat.roughness = 0.0;
        return mat;
    }

    pub fn glossy(color: Vector3<f32>) -> Self {
        let mut mat: Self = Self::default();
        mat.albedo = [color.x, color.y, color.z, 1.0];
        mat.roughness = 0.2;
        mat.metallic = 0.2;
        return mat;
    }

    pub fn darkest_stone() -> Self {
        let mut mat: Self = Self::default();
        mat.albedo = [0.05, 0.05, 0.05, 1.0];
        mat.metallic = 0.0;
        mat.roughness = 0.5;
        return mat;
    }

    pub fn dark_stone() -> Self {
        let mut mat: Self = Self::default();
        mat.albedo = [0.07, 0.07, 0.07, 1.0];
        mat.metallic = 0.0;
        mat.roughness = 0.7;
        return mat;
    }
}

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

pub struct Model {
    pub meshes: Vec<Mesh>,
}
