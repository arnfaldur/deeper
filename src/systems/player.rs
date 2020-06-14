use specs::prelude::*;

use std::f32::consts::PI;

use cgmath::{
    prelude::*,
    num_traits::clamp,
    Vector2, Vector3, Vector4
};

use crate::input::{InputState, Key};
use crate::components::*;
use crate::graphics::{Context, project_screen_to_world, to_pos3, correction_matrix};

pub struct CameraControlSystem;

impl<'a> System <'a> for CameraControlSystem {
    type SystemData = (
        ReadExpect<'a, InputState>,
        ReadExpect<'a, Player>,
        ReadExpect<'a, PlayerCamera>,
        WriteStorage<'a, Camera>,
        ReadStorage<'a, Position3D>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, SphericalOffset>,
        WriteStorage<'a, Destination>,
    );

    fn run(&mut self, (input, player, player_cam, mut cam, pos3d, mut pos, mut offset, mut dests): Self::SystemData) {

        // Should these be a feature of the spherical offset?
        const MINIMUM_PHI : f32 = 0.15  * PI;
        const MAXIMUM_PHI : f32 = 0.325 * PI;

        const MINIMUM_RADIUS : f32 = 10.0;
        const MAXIMUM_RADIUS : f32 = 25.0;

        let mut camera = cam.get_mut(player_cam.entity).unwrap();
        let mut cam_offset = offset.get_mut(player_cam.entity).unwrap();

        // Zoom controls
        cam_offset.radius += -input.mouse.scroll * cam_offset.radius_delta;
        cam_offset.radius = clamp(cam_offset.radius, MINIMUM_RADIUS, MAXIMUM_RADIUS);

        cam_offset.phi = (cam_offset.radius - MINIMUM_RADIUS) / (MAXIMUM_RADIUS - MINIMUM_RADIUS) * (MAXIMUM_PHI - MINIMUM_PHI) + MINIMUM_PHI;

        // camera orbiting system enabled for now
        if input.mouse.middle.down {
            let mouse_delta = input.mouse.delta();

            cam_offset.theta += cam_offset.theta_delta * mouse_delta.x;
        }

        let mut cam_pos = pos.get_mut(player_cam.entity).unwrap();
        let cam_3d_pos = pos3d.get(player_cam.entity).unwrap();

        let to_center = (cam_pos.to_vec3() - cam_3d_pos.0).normalize() * 0.02;
        let cam_front = Vector2::new(to_center.x,  to_center.y);
        let cam_right = Vector2::new(to_center.y, -to_center.x);

        if input.is_key_down(Key::E) {
            cam_pos.0 += cam_front;
            camera.roaming = true;
        }
        if input.is_key_down(Key::S) {
            cam_pos.0 -= cam_right;
            camera.roaming = true;
        }
        if input.is_key_down(Key::D) {
            cam_pos.0 -= cam_front;
            camera.roaming = true;
        }
        if input.is_key_down(Key::F) {
            cam_pos.0 += cam_right;
            camera.roaming = true;
        }

        // Need to deal with removing the destination also
        if !camera.roaming {
            dests.insert(player_cam.entity, Destination(pos.get(player.entity).unwrap().0));
        }
    }
}

pub struct PlayerSystem;
// Note(Jökull): Is this really just the input handler?
impl<'a> System<'a> for PlayerSystem {
    type SystemData = (
        Entities<'a>,
        Read<'a, LazyUpdate>,
        WriteStorage<'a, Orientation>,
        WriteStorage<'a, Destination>,
        ReadExpect<'a, InputState>,
        ReadExpect<'a, Context>,
        ReadExpect<'a, Player>,
        ReadExpect<'a, PlayerCamera>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Position3D>,
        WriteStorage<'a, Camera>,
        ReadStorage<'a, Faction>,
        ReadStorage<'a, HitPoints>,
        ReadStorage<'a, DynamicBody>,
    );

    fn run(&mut self, (ents, updater, mut orient, mut dest, input, context, player, player_cam, pos, pos3d, mut cam, faction, hp, dynamic): Self::SystemData) {


        let player_pos = pos.get(player.entity)
            .expect("I have no place in this world.");
        let mut player_orient = orient.get_mut(player.entity)
            .expect("We have no direction in life.");

        let mouse_pos = input.mouse.pos;

        // Click to move around
        // Note(Jökull): We need to make this prettier
        if input.mouse.left.down {
            let mut camera = cam.get_mut(player_cam.entity).unwrap();
            let camera_3d_pos = pos3d.get(player_cam.entity).unwrap();
            let camera_pos = pos.get(player_cam.entity).unwrap();

            let aspect_ratio = context.sc_desc.width as f32 / context.sc_desc.height as f32;

            let mx_view = cgmath::Matrix4::look_at(
                to_pos3(camera_3d_pos.0),
                to_pos3(camera_pos.to_vec3()),
                cgmath::Vector3::unit_z(),
            );
            let mx_projection = cgmath::perspective(
                cgmath::Deg(camera.fov),
                aspect_ratio,
                1.0,
                1000.0,
            );

            if let Some(mouse_world_pos) = project_screen_to_world(
                Vector3::new(mouse_pos.x, mouse_pos.y, 1.0),
                correction_matrix() * mx_projection * mx_view,
                Vector4::new(0, 0, context.sc_desc.width as i32, context.sc_desc.height as i32),
            ) {
                let ray_delta: Vector3<f32> = mouse_world_pos - camera_3d_pos.0;
                let t: f32 = mouse_world_pos.z / ray_delta.z;
                let ray_hit = (mouse_world_pos - ray_delta * t).truncate();

                dest.insert(player.entity, Destination(ray_hit));
                camera.roaming = false;

                let difference: Vector2<f32> = (ray_hit - player_pos.0).normalize();

                let mut new_rotation = (difference.y / difference.x).atan() / PI * 180.0;
                if difference.x > 0.0 {
                    new_rotation += 180.0;
                }
                (player_orient.0).0 = new_rotation;
            }
        }

        if input.is_key_pressed(Key::Space) {
            for (ent, pos, &HitPoints { max, health }, &faction, dynamic) in (&ents, &pos, &hp, &faction, &dynamic).join() {
                let forward_vector = cgmath::Basis2::<f32>::from_angle(player_orient.0).rotate_vector(-Vector2::unit_x());
                let in_front = (pos.0 - player_pos.0).normalize().dot(forward_vector.normalize()) > 0.5;
                if faction == Faction::Enemies && pos.0.distance(player_pos.0) < 2.0 && in_front {
                    updater.insert(ent, HitPoints { max, health: (health - 1.0).max(0.0) });
                    updater.insert(ent, Velocity((pos.0 - player_pos.0).normalize() * 1.5 / dynamic.0));
                }
            }
        }
    }
}
