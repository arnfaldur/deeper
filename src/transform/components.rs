use cgmath::{Array, Deg, Matrix4, Quaternion, Rotation3, SquareMatrix, Vector2, Vector3, Zero};

pub struct Position(pub Vector2<f32>);

impl From<&Position> for Position3D {
    fn from(pos: &Position) -> Self { Position3D(pos.0.extend(0.)) }
}

impl From<&Position> for Vector3<f32> {
    fn from(pos: &Position) -> Self { pos.0.extend(0.) }
}

pub struct Position3D(pub Vector3<f32>);

pub struct Rotation(pub Deg<f32>);

impl From<&Rotation> for Rotation3D {
    fn from(rot: &Rotation) -> Self { Rotation3D(Quaternion::from_angle_z(rot.0)) }
}

pub struct Rotation3D(pub Quaternion<f32>);

pub struct Scale(pub f32);

pub struct NonUniformScale(pub Vector3<f32>);

impl From<&Position> for Matrix4<f32> {
    fn from(pos: &Position) -> Self { Matrix4::from_translation(pos.0.extend(0.)) }
}

impl From<&Position3D> for Matrix4<f32> {
    fn from(pos: &Position3D) -> Self { Matrix4::from_translation(pos.0) }
}

impl From<&Rotation> for Matrix4<f32> {
    fn from(rot: &Rotation) -> Self { Matrix4::from_angle_z(rot.0) }
}

impl From<&Rotation3D> for Matrix4<f32> {
    fn from(rot: &Rotation3D) -> Self { Matrix4::from(rot.0) }
}

impl From<&Scale> for Matrix4<f32> {
    fn from(scale: &Scale) -> Self { Matrix4::from_scale(scale.0) }
}

impl From<&NonUniformScale> for Matrix4<f32> {
    fn from(scale: &NonUniformScale) -> Self {
        Matrix4::from_nonuniform_scale(scale.0.x, scale.0.y, scale.0.z)
    }
}

pub struct RelativeTransform(pub Matrix4<f32>);

pub struct AbsoluteTransform(pub Matrix4<f32>);

pub struct Transform {
    position: Vector3<f32>,
    rotation: Quaternion<f32>,
    scale: Vector3<f32>,
    relative: Matrix4<f32>,
    global: Matrix4<f32>,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vector3::zero(),
            rotation: Quaternion::zero(),
            scale: Vector3::from_value(1.0),
            relative: Matrix4::identity(),
            global: Matrix4::identity(),
        }
    }
}

impl Transform {
    pub fn new(position: Vector3<f32>, rotation: Quaternion<f32>, scale: Vector3<f32>) -> Self {
        let relative = Matrix4::from_translation(position)
            * Matrix4::from(rotation)
            * Matrix4::from_diagonal(scale.extend(0.));
        Self {
            position,
            rotation,
            scale,
            relative,
            global: Matrix4::identity(),
        }
    }
}
