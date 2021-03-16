use std::collections::HashSet;

use cgmath::{Deg, Euler, Matrix4, Quaternion, Rad, Rotation3, SquareMatrix, Vector3, Zero};
use legion::Entity;

pub struct Parent(pub Entity);

pub struct Children(pub HashSet<Entity>);

pub struct Position(pub Vector3<f32>);

pub struct Rotation(pub Quaternion<f32>);

pub struct Scale(pub f32);

impl From<&Position> for Matrix4<f32> {
    fn from(pos: &Position) -> Self { Matrix4::from_translation(pos.0) }
}

impl From<&Rotation> for Matrix4<f32> {
    fn from(rot: &Rotation) -> Self { Matrix4::from(rot.0) }
}

impl From<&Scale> for Matrix4<f32> {
    fn from(scale: &Scale) -> Self { Matrix4::from_scale(scale.0) }
}

impl Rotation {
    pub fn to_rad(&self) -> Rad<f32> { Euler::from(self.0).z }
    pub fn to_deg(&self) -> Deg<f32> { Euler::from(self.0).z.into() }
    pub fn from_deg(deg: f32) -> Self { Self(Quaternion::from_angle_z(Deg(deg))) }
    pub fn from_rad(deg: f32) -> Self { Self(Quaternion::from_angle_z(Rad(deg))) }
}

impl From<Rad<f32>> for Rotation {
    fn from(rad: Rad<f32>) -> Self { Self(Quaternion::from_angle_z(rad)) }
}

impl From<Deg<f32>> for Rotation {
    fn from(deg: Deg<f32>) -> Self { Self(Quaternion::from_angle_z(deg)) }
}

#[derive(Copy, Clone)]
pub struct Transform {
    pub absolute: Matrix4<f32>,
    pub relative: Matrix4<f32>,
}

impl Transform {
    pub fn identity() -> Self {
        Transform {
            absolute: Matrix4::identity(),
            relative: Matrix4::identity(),
        }
    }
}

impl Default for Position {
    fn default() -> Self {
        println!("Creating a default Position");
        return Position(Vector3::zero());
    }
}

pub struct SphericalOffset {
    pub phi: f32,
    pub theta: f32,
    pub radius: f32,
    pub theta_delta: f32,
    pub phi_delta: f32,
    pub radius_delta: f32,
}

impl SphericalOffset {
    pub fn camera_offset() -> Self {
        use std::f32::consts::PI;
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
