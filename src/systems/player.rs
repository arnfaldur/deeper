extern crate cgmath;

use specs::prelude::*;
use std::f32::consts::PI;

use cgmath::{prelude::*, Vector2, Vector3};

use crate::input::{InputState, Key};
use crate::components::*;

use self::cgmath::{Vector4};
use crate::graphics::{Context, project_screen_to_world, to_pos3, correction_matrix};

pub struct PlayerSystem;

// Note(Jökull): Is this really just the input handler?
impl<'a> System<'a> for PlayerSystem {
    type SystemData = (
        Entities<'a>,
        Read<'a, LazyUpdate>,
        WriteStorage<'a, Orientation>,
        WriteStorage<'a, Destination>,
        WriteStorage<'a, SphericalOffset>,
        ReadExpect<'a, InputState>,
        ReadExpect<'a, Context>,
        ReadExpect<'a, Player>,
        ReadExpect<'a, PlayerCamera>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Position3D>,
        ReadStorage<'a, Camera>,
        ReadStorage<'a, Faction>,
        ReadStorage<'a, HitPoints>,
        ReadStorage<'a, DynamicBody>,
    );

    fn run(&mut self, (ents, updater, mut orient, mut dest, mut offset, input, context, player, player_cam, pos, pos3d, cam, faction, hp, dynamic): Self::SystemData) {
        let camera = cam.get(player_cam.0).unwrap();
        let camera_pos = pos3d.get(player_cam.0).unwrap();
        let mut camera_offset = offset.get_mut(player_cam.0).unwrap();

        let mouse_pos = input.mouse.pos;
        let mouse_delta = input.mouse.pos - input.mouse.last_pos;

        // camera orbiting system enabled for now
        if input.mouse.middle.down {
            camera_offset.theta += camera_offset.theta_delta * mouse_delta.x;
            camera_offset.phi += camera_offset.phi_delta * mouse_delta.y;
            camera_offset.phi = camera_offset.phi.max(0.1 * PI).min(0.25 * PI);
        }

        let player_pos = pos.get(player.entity)
            .expect("I have no place in this world.");
        let mut player_orient = orient.get_mut(player.entity)
            .expect("We have no direction in life.");

        if input.mouse.left.down {
            // Note(Jökull): We need a better solution for this

            let mx_view = cgmath::Matrix4::look_at(
                to_pos3(camera_pos.0),
                to_pos3(player_pos.to_vec3()),
                cgmath::Vector3::unit_z(),
            );
            let mx_projection = cgmath::perspective(
                cgmath::Deg(camera.fov),
                1920f32 / 1080f32,
                1.0,
                1000.0,
            );

            if let Some(mouse_world_pos) = project_screen_to_world(
                Vector3::new(mouse_pos.x, mouse_pos.y, 1.0),
                correction_matrix() * mx_projection * mx_view,
                Vector4::new(0, 0, context.sc_desc.width as i32, context.sc_desc.height as i32),
            ) {
                let ray_delta: Vector3<f32> = mouse_world_pos - camera_pos.0;
                let t: f32 = mouse_world_pos.z / ray_delta.z;
                let ray_hit = (mouse_world_pos - ray_delta * t).truncate();

                dest.insert(player.entity, Destination(ray_hit));

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
                let in_frontness = (pos.0 - player_pos.0).normalize().dot(forward_vector.normalize());
                if faction == Faction::Enemies && pos.0.distance(player_pos.0) < 2.0 && in_frontness > 0.5 {
                    updater.insert(ent, HitPoints { max, health: (health - 1.0).max(0.0) });
                    updater.insert(ent, Velocity((pos.0 - player_pos.0).normalize() * 1.5 / dynamic.0));
                }
            }
        }
    }
}
