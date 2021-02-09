mod entity_builder;

extern crate cgmath;

use std::f32::consts::PI;

use cgmath::{Deg, Matrix4, Vector2, Vector3, Zero};
use legion::Entity;
use nphysics2d::object::{DefaultBodyHandle, DefaultColliderHandle};

use crate::graphics;

// Note(Jökull): Begin entity pointers
pub struct Player {
    pub entity: Entity,
}

pub struct ActiveCamera {
    pub entity: Entity,
}

pub struct PlayerCamera {
    pub entity: Entity,
}

pub struct Parent(pub Entity);

// end entity pointers

pub struct FrameTime(pub f32);

#[derive(Debug)]
#[derive(Copy, Clone)]
pub struct Position(pub Vector2<f32>);

impl Position {
    pub fn to_vec3(self) -> Vector3<f32> { Vector3::new(self.0.x, self.0.y, 0.0) }
}

impl Into<Vector3<f32>> for Position {
    fn into(self) -> Vector3<f32> { return self.0.extend(0.0); }
}

impl Into<Vector3<f32>> for &Position {
    fn into(self) -> Vector3<f32> { return self.0.extend(0.0); }
}

#[derive(Debug)]
pub struct Velocity(pub Vector2<f32>);

impl Velocity {
    pub fn new() -> Velocity { Velocity(Vector2::new(0.0, 0.0)) }
}

pub struct Force(pub nphysics2d::algebra::Force2<f32>);
impl Default for Force {
    fn default() -> Self { Force(nphysics2d::algebra::Force2::zero()) }
}

pub struct Orientation(pub Deg<f32>);

pub struct Speed(pub f32);

pub struct Acceleration(pub f32);

pub struct DisabledBody;

pub struct StaticBody;

pub struct DynamicBody {
    pub mass: f32,
}

pub struct CircleCollider {
    pub radius: f32,
}

pub struct SquareCollider {
    pub side_length: f32,
}

pub struct BodyHandle(pub DefaultBodyHandle);
pub struct ColliderHandle(pub DefaultColliderHandle);

pub struct Agent;

pub struct AIFollow {
    pub target: Entity,
    pub minimum_distance: f32,
}

pub struct Destination {
    pub goal: Vector2<f32>,
    pub next: Vector2<f32>,
}

impl Destination {
    pub fn simple(goal: Vector2<f32>) -> Destination {
        Destination {
            goal,
            next: Vector2 { x: 0., y: 0. },
        }
    }
}

#[derive(Eq, PartialEq)]
#[derive(Copy, Clone)]
pub enum Faction {
    Enemies,
    Friends,
}

pub struct HitPoints {
    pub max: f32,
    pub health: f32,
}

#[derive(Copy, Clone)]
pub enum MapTransition {
    None,
    Deeper, // Down to the next floor
}

pub struct MapSwitcher(pub MapTransition);

pub struct GuiWindow {
    pub size: [f32; 2],
    pub name: String,
    pub view: Box<dyn FnOnce(&imgui::Ui)>,
}

pub struct Camera {
    pub fov: f32,
    pub up: Vector3<f32>,
    pub roaming: bool,
}

pub struct Target(pub Entity);

pub struct Position3D(pub Vector3<f32>);

pub struct SphericalOffset {
    pub phi: f32,
    pub theta: f32,
    pub radius: f32,
    pub theta_delta: f32,
    pub phi_delta: f32,
    pub radius_delta: f32,
}

impl SphericalOffset {
    pub fn new() -> Self {
        Self {
            phi: 0.0,
            theta: 0.0,
            radius: 1.0,
            theta_delta: 0.0,
            phi_delta: 0.0,
            radius_delta: 0.0,
        }
    }

    pub fn camera_offset() -> Self {
        Self {
            phi: 0.2 * PI,
            theta: PI / 3.0,
            radius: 15.0,
            // TODO: Not satisfactory, but need to limit untraceable magic constants
            theta_delta: -0.005,
            phi_delta: 0.0025,
            radius_delta: 0.3,
        }
    }
}

pub struct StaticModel {
    pub idx: usize,
    pub bind_group: wgpu::BindGroup,
}

impl StaticModel {
    pub fn new(
        context: &graphics::Context,
        idx: usize,
        offset: Vector3<f32>,
        scale: f32,
        z_rotation: f32,
        material: graphics::data::Material,
    ) -> Self {
        let uniforms_size = std::mem::size_of::<graphics::data::LocalUniforms>() as u64;

        let mut matrix = Matrix4::from_scale(scale);
        matrix = Matrix4::from_angle_z(cgmath::Deg(z_rotation)) * matrix;
        matrix = Matrix4::from_translation(offset) * matrix;

        let local_uniforms = graphics::data::LocalUniforms {
            model_matrix: matrix.into(),
            material,
        };

        let (_uniform_buf, bind_group) = context.model_bind_group_from_uniform_data(local_uniforms);

        Self { idx, bind_group }
    }
}

pub struct Model3D {
    pub idx: usize,
    pub offset: Vector3<f32>,
    pub scale: f32,
    pub z_rotation: f32,
    pub material: graphics::data::Material,

    pub bind_group: wgpu::BindGroup,
    pub uniform_buffer: wgpu::Buffer,

    // TODO: Move out of Model3D
    pub local_uniforms_cache: graphics::data::LocalUniforms,
}

// Note(Jökull): Probably not great to have both constructor and builder patterns
impl Model3D {
    pub fn new(context: &graphics::Context) -> Self {
        let uniforms_size = std::mem::size_of::<graphics::data::LocalUniforms>() as u64;

        let (uniform_buffer, bind_group) =
            context.model_bind_group_from_uniform_data(graphics::data::LocalUniforms::new());

        Self {
            idx: 0,
            offset: Vector3::new(0.0, 0.0, 0.0),
            material: graphics::data::Material::default(),
            bind_group,
            scale: 1.0,
            z_rotation: 0.0,
            uniform_buffer,
            local_uniforms_cache: graphics::data::LocalUniforms::new(),
        }
    }

    pub fn from_index(context: &graphics::Context, index: usize) -> Self {
        let mut m = Self::new(context);
        m.idx = index;
        return m;
    }

    pub fn with_offset(mut self, offset: Vector3<f32>) -> Self {
        self.offset = offset;
        self
    }

    pub fn with_scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }

    pub fn with_z_rotation(mut self, z_rotation: f32) -> Self {
        self.z_rotation = z_rotation;
        self
    }

    pub fn with_material(mut self, material: graphics::data::Material) -> Self {
        self.material = material;
        self
    }
}

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum TileType {
    Wall(Option<WallDirection>),
    Floor,
    Path,
    Nothing,
    LadderDown,
}

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum WallDirection {
    North,
    West,
    South,
    East,
}

pub struct FloorNumber(pub i32);
