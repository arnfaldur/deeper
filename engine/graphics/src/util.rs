#![allow(unused)]

// Series of utility functions that solve miscellaneous but specific problems

use cgmath::num_traits::Float;
use cgmath::{BaseFloat, Deg, EuclideanSpace};

use crate::data::{DirectionalLight, Lights, PointLight};
use crate::{GraphicsContext, MAX_NR_OF_POINT_LIGHTS};

pub fn sc_desc_from_size(size: winit::dpi::PhysicalSize<u32>) -> wgpu::SwapChainDescriptor {
    wgpu::SwapChainDescriptor {
        usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
        format: crate::COLOR_FORMAT,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Mailbox,
    }
}

pub fn generate_matrix(aspect_ratio: f32, t: f32) -> cgmath::Matrix4<f32> {
    let mx_projection = cgmath::perspective(Deg(45.0), aspect_ratio, 1.0, 10.0);
    let mx_view = cgmath::Matrix4::look_at_rh(
        cgmath::Point3::new(5. * t.cos(), 5.0 * t.sin(), 3.),
        cgmath::Point3::new(0., 0., 0.),
        cgmath::Vector3::unit_z(),
    );

    correction_matrix() * mx_projection * mx_view
}

// Function by Vallentin
// https://vallentin.dev/2019/08/12/screen-to-world-cgmath
pub fn project_screen_to_world(
    screen: cgmath::Vector3<f32>,
    view_projection: cgmath::Matrix4<f32>,
    viewport: cgmath::Vector4<f32>,
) -> Option<cgmath::Vector3<f32>> {
    use cgmath::SquareMatrix;
    if let Some(inv_view_projection) = view_projection.invert() {
        let world = cgmath::Vector4::new(
            (screen.x - viewport.x) / viewport.z as f32 * 2.0 - 1.0,
            // Screen Origin is Top Left    (Mouse Origin is Top Left)
            // (screen.y - (viewport.y as f32)) / (viewport.w as f32) * 2.0 - 1.0,
            // Screen Origin is Bottom Left (Mouse Origin is Top Left)
            (1.0 - (screen.y - viewport.y) / viewport.w) * 2.0 - 1.0,
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

pub fn generate_view_matrix(
    cam: &crate::components::Camera,
    cam_pos: cgmath::Vector3<f32>,
    cam_target: cgmath::Vector3<f32>,
    aspect_ratio: f32,
) -> cgmath::Matrix4<f32> {
    let mx_view = cgmath::Matrix4::look_at_rh(
        cgmath::Point3::from_vec(cam_pos),
        cgmath::Point3::from_vec(cam_target),
        cgmath::Vector3::unit_z(),
    );

    let mx_perspective = cgmath::perspective(cgmath::Deg(cam.fov), aspect_ratio, 1.0, 1000.0);

    correction_matrix() * mx_perspective * mx_view
}

pub fn generate_ortho_matrix(size: winit::dpi::PhysicalSize<f32>) -> cgmath::Matrix4<f32> {
    let mx_ortho = cgmath::ortho(0.0, size.width, size.height, 0.0, 0.0, 1.0);
    correction_matrix() * mx_ortho
}

#[rustfmt::skip]
pub fn correction_matrix() -> cgmath::Matrix4<f32> {
    cgmath::Matrix4::new(
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 0.5, 0.0,
        0.0, 0.0, 0.5, 1.0,
    )
}
