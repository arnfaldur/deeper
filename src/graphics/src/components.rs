use std::sync::Arc;

use cgmath::{Matrix4, Vector3};
use entity_smith::EntitySmith;

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
    pub idx: usize,
    pub scale: f32,
    pub material: crate::data::Material,
}

// Note(Jökull): Probably not great to have both constructor and builder patterns
impl Model3D {
    pub fn new() -> Self {
        Self {
            idx: 0,
            material: crate::data::Material::default(),
            scale: 1.0,
        }
    }

    pub fn from_index(index: usize) -> Self {
        let mut m = Self::new();
        m.idx = index;
        return m;
    }

    pub fn with_scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }

    pub fn with_material(mut self, material: crate::data::Material) -> Self {
        self.material = material;
        self
    }
}

#[derive(Clone)]
pub struct StaticModel {
    pub idx: usize,
    pub bind_group: Arc<wgpu::BindGroup>,
}

impl StaticModel {
    pub fn new(
        context: &crate::GraphicsContext,
        idx: usize,
        offset: Vector3<f32>,
        scale: f32,
        z_rotation: f32,
        material: crate::data::Material,
    ) -> Self {
        let _uniforms_size = std::mem::size_of::<crate::data::LocalUniforms>() as u64;

        let mut matrix = Matrix4::from_scale(scale);
        matrix = Matrix4::from_angle_z(cgmath::Deg(z_rotation)) * matrix;
        matrix = Matrix4::from_translation(offset) * matrix;

        let local_uniforms = crate::data::LocalUniforms::new(matrix.into(), material);

        let bind_group = context.model_bind_group_from_uniform_data(local_uniforms);

        Self {
            idx,
            bind_group: Arc::new(bind_group),
        }
    }

    pub fn from_uniforms(
        context: &crate::GraphicsContext,
        idx: usize,
        local_uniforms: crate::data::LocalUniforms,
    ) -> Self {
        let bind_group = context.model_bind_group_from_uniform_data(local_uniforms);

        Self {
            idx,
            bind_group: Arc::new(bind_group),
        }
    }
}
