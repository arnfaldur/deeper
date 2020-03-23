use raylib::prelude::*;
use specs::prelude::*;
use std::f32::consts::PI;
use std::ops::Mul;

use crate::components::components::*;

pub struct MovementSystem;

impl<'a> System<'a> for MovementSystem {
    type SystemData = (WriteStorage<'a, Position>, ReadStorage<'a, Velocity>);

    fn run(&mut self, (mut pos, vel): Self::SystemData) {
        for (pos, vel) in (&mut pos, &vel).join() {
            pos.0 += vel.0;
        }
    }
}

pub struct SphericalFollowSystem;

impl<'a> System<'a> for SphericalFollowSystem {
    type SystemData = (
        ReadStorage<'a, Position>,
        ReadStorage<'a, Target>,
        ReadStorage<'a, SphericalOffset>,
        WriteStorage<'a, Position3D>,
    );

    fn run(&mut self, (pos2d, target, follow, mut pos3d): Self::SystemData) {
        for (target, follow, pos3d) in (&target, &follow, &mut pos3d).join() {
            pos3d.0 = pos2d.get(target.0).cloned().unwrap().to_vec3();
            pos3d.0.x += follow.radius * follow.theta.cos() * follow.phi.cos();
            pos3d.0.y += follow.radius * follow.theta.sin() * follow.phi.cos();
            pos3d.0.z += follow.radius * follow.phi.sin();
        }
    }
}

struct CameraSystem;

impl<'a> System<'a> for CameraSystem {
    type SystemData = (
        WriteStorage<'a, crate::components::components::Camera>,
        ReadStorage<'a, Position3D>,
    );

    fn run(&mut self, (camera, pos): Self::SystemData) {}
}

extern crate raylib;

use raylib::shaders::Shader;

pub struct GraphicsSystem {
    pub thread: RaylibThread,
    pub model_array: Vec<Model>,
    pub l_shader: Shader,
    mat_model_loc: i32,
    eye_position_loc: i32,
}

impl GraphicsSystem {
    pub fn new(thread: RaylibThread, model_array: Vec<Model>, l_shader: Shader) -> Self {
        Self {
            thread,
            model_array,
            l_shader,
            mat_model_loc: 0,
            eye_position_loc: 0,
        }
    }
}

impl<'a> System<'a> for GraphicsSystem {
    type SystemData = (
        WriteExpect<'a, RaylibHandle>,
        ReadExpect<'a, ActiveCamera>,
        ReadStorage<'a, crate::components::components::Camera>,
        ReadStorage<'a, Target>,
        ReadStorage<'a, Position3D>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Model3D>,
    );

    fn run(&mut self, (mut rl, active_cam, camera, target, pos3d, pos, models): Self::SystemData) {
        let fps = 1.0 / rl.get_frame_time();
        let mut d2: RaylibDrawHandle = rl.begin_drawing(&self.thread);

        d2.clear_background(Color::BLACK);

        {
            let active_camera = camera.get(active_cam.0).unwrap();
            let active_target = target.get(active_cam.0).unwrap();
            let camera_position = pos3d.get(active_cam.0).unwrap().0;

            self.l_shader
                .set_shader_value(self.eye_position_loc, camera_position);

            let mut d3 = d2.begin_mode_3D(Camera3D::perspective(
                camera_position,
                pos.get(active_target.0).unwrap().to_vec3(),
                active_camera.up,
                active_camera.fov,
            ));

            for (pos, model) in (&pos, &models).join() {
                let model_pos = pos.clone().to_vec3() + model.offset;

                self.l_shader.set_shader_value_matrix(
                    self.mat_model_loc,
                    Matrix::scale(model.scale, model.scale, model.scale)
                        .mul(Matrix::rotate(
                            Vector3::new(0.0, 0.0, 1.0),
                            PI * model.z_rotation / 180.0,
                        ))
                        .mul(Matrix::translate(model_pos.x, model_pos.y, model_pos.z)),
                );

                d3.draw_model_ex(
                    &self.model_array[model.idx],
                    model_pos,
                    Vector3::new(0.0, 0.0, 1.0),
                    model.z_rotation,
                    Vector3::new(model.scale, model.scale, model.scale),
                    model.tint,
                );
            }
        }

        d2.draw_text("deeper", 12, 12, 30, Color::WHITE);
        d2.draw_text(&format!("FPS {}", fps), 12, 46, 18, Color::WHITE);
    }

    fn setup(&mut self, world: &mut World) {
        self.mat_model_loc = self.l_shader.get_shader_location("matModel");
        self.eye_position_loc = self.l_shader.get_shader_location("eyePosition");

        println!("GraphicsSystem setup!");
    }
}

pub struct PlayerSystem {
    // Note(Jökull): Yeah, I know. This is just while we're feeling out what is the
    //               responsibility of the input handling system exactly
    last_mouse_pos: Vector2,
}

impl PlayerSystem {
    pub fn new() -> Self {
        Self {
            last_mouse_pos: Vector2::zero(),
        }
    }
}

// Note(Jökull): Is this really just the input handler?
impl<'a> System<'a> for PlayerSystem {
    type SystemData = (
        ReadExpect<'a, RaylibHandle>,
        ReadExpect<'a, Player>,
        ReadExpect<'a, PlayerCamera>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Position3D>,
        ReadStorage<'a, crate::components::components::Camera>,
        WriteStorage<'a, Model3D>,
        WriteStorage<'a, Velocity>,
        WriteStorage<'a, SphericalOffset>,
    );

    fn run(
        &mut self,
        (rl, player, player_cam, pos, pos3d, cam, mut model, mut vel, mut offset): Self::SystemData,
    ) {
        use raylib::consts::{KeyboardKey::*, MouseButton::*};
        let camera = cam.get(player_cam.0).unwrap();
        let camera_pos = pos3d.get(player_cam.0).unwrap();
        let mut camera_offset = offset.get_mut(player_cam.0).unwrap();

        let mouse_delta = rl.get_mouse_position() - self.last_mouse_pos;
        self.last_mouse_pos = rl.get_mouse_position();

        if rl.is_mouse_button_down(MOUSE_MIDDLE_BUTTON) {
            camera_offset.theta += camera_offset.theta_delta * mouse_delta.x;
            camera_offset.phi += camera_offset.phi_delta * mouse_delta.y;
            camera_offset.phi = camera_offset.phi.max(0.1 * PI).min(0.25 * PI);
        }

        let mut player_vel = vel.get_mut(player.entity).unwrap();
        player_vel.0 = vec2(0.0, 0.0);

        if rl.is_mouse_button_down(MOUSE_LEFT_BUTTON) {
            // Note(Jökull): We need a better solution for this
            let player_pos = pos.get(player.entity).unwrap();
            let rl_cam = raylib::camera::Camera::perspective(
                camera_pos.0,
                player_pos.to_vec3(),
                camera.up,
                camera.fov,
            );
            let mouse_ray = rl.get_mouse_ray(rl.get_mouse_position(), rl_cam);
            let t = mouse_ray.position.z / mouse_ray.direction.z;
            let ray_hit = mouse_ray.position - mouse_ray.direction.scale_by(t);
            let difference = ray_hit - player_pos.to_vec3();
            let difference = difference.scale_by(1.0 / difference.length());
            player_vel.0.x = difference.x * player.speed;
            player_vel.0.y = difference.y * player.speed;

            let model = model.get_mut(player.entity).unwrap();
            let mut new_rotation = (difference.y / difference.x).atan() / PI * 180.0;
            if difference.x > 0.0 {
                new_rotation += 180.0;
            }
            model.z_rotation = new_rotation;
        }
    }

    fn setup(&mut self, world: &mut World) {
        println!("PlayerSystem setup!");
    }
}

pub struct Physics2DSystem;

impl<'a> System<'a> for Physics2DSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Position>,
        WriteStorage<'a, Velocity>,
        ReadStorage<'a, StaticBody>,
        ReadStorage<'a, DynamicBody>,
        ReadStorage<'a, CircleCollider>,
        ReadStorage<'a, SquareCollider>,
    );

    fn run(&mut self, (ents, pos, mut vel, statics, dynamics, circles, squares): Self::SystemData) {
        for _ in (&dynamics, &statics).join() {
            panic!("There's a naughty static body that really feels dynamic inside.");
        }

        for (_, pos_d, vel, circle_d) in (&dynamics, &pos, &mut vel, &circles).join() {
            for (_, pos_s, circle_s) in (&statics, &pos, &circles).join() {
                if (pos_d.0 + vel.0).distance_to(pos_s.0) < (circle_d.radius + circle_s.radius) {
                    let diff = pos_s.0 - pos_d.0;
                    let collinear_part = diff.scale_by(vel.0.dot(diff));
                    vel.0 -= collinear_part;
                }
            }
            for (_, pos_s, square_s) in (&statics, &pos, &squares).join() {
                let thing: Vector2 = pos_d.0 - pos_s.0;
                if (pos_d.0 + vel.0).distance_to(pos_s.0) < (circle_d.radius + square_s.side_length) {
                    let diff = pos_s.0 - pos_d.0;
                    let collinear_part = diff.scale_by(vel.0.dot(diff));
                    vel.0 -= collinear_part;
                }
            }
        }

        // let boi: Entity;
        // let igi: Position;
        // for (entity_a, pos_a, stat, circle_a) in (&ents, &pos, &statics, &circles).join() {
        //     for (entity_b, pos_b, dynamic, circle_b) in (&ents, &pos, &dynamics, &circles).join() {
        //         if entity_a != entity_b && (pos_a.0.distance_to(pos_b.0) < (circle_a.radius + circle_b.radius)) {
        //             println!("{:?} is colliding with {:?}!", entity_a, entity_b);
        //         }
        //     }
        // }
    }
}

use crate::dung_gen::{DungGen, WallDirection};

pub struct DunGenSystem {
    pub dungeon: DungGen,
}

impl<'a> System<'a> for DunGenSystem {
    type SystemData = ();

    fn run(&mut self, (): Self::SystemData) {}

    fn setup(&mut self, world: &mut World) {
        use crate::dung_gen::WallType;

        for x in 0..=self.dungeon.width {
            for y in 0..=self.dungeon.height {
                match self.dungeon.world.get(&(x, y)) {
                    None => (),
                    Some(WallType::NOTHING) => (),
                    Some(WallType::FLOOR) => {
                        world.create_entity()
                            .with(Position(vec2(x as f32, y as f32)))
                            .with(FloorTile)
                            .with(Model3D::from_index(1).with_tint(Color::DARKGRAY))
                            .build();
                    }
                    Some(WallType::WALL(maybe_direction)) => {
                        world.create_entity()
                            .with(Position(vec2(x as f32, y as f32)))
                            .with(WallTile)
                            .with(
                                match maybe_direction {
                                    None => Model3D::from_index(0).with_tint(Color::LIGHTGRAY),
                                    Some(direction) => Model3D::from_index(4).
                                        with_tint(Color::DARKGRAY).with_z_rotation(
                                        match direction {
                                            WallDirection::North => 0.0,
                                            WallDirection::South => 180.0,
                                            WallDirection::East => 270.0,
                                            WallDirection::West => 90.0,
                                        }
                                    ),
                                }
                            )
                            .with(StaticBody)
                            .with(SquareCollider { side_length: 0.5 })
                            .build();
                    }
                }
            }
        }
    }
}
