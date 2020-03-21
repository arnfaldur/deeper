extern crate specs;

use specs::{
    DispatcherBuilder,
    WorldExt,
    Builder,
    System,
    AccessorCow,
    RunningTime,
    Component,
    VecStorage,
};

use raylib::prelude::*;
use std::f32::consts::PI;
use rand::seq::SliceRandom;
use std::ops::{Add, Mul};
use std::process::exit;
use specs::prelude::*;


#[derive(Component, Debug, Copy, Clone)]
#[storage(VecStorage)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

impl Position {
    pub fn new() -> Position { Position { x: 0.0, y: 0.0 } }
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

impl Velocity {
    pub fn new() -> Velocity { Velocity { x: 0.0, y: 0.0 } }
}

#[derive(Component)]
#[storage(VecStorage)]
pub struct Agent;

pub struct MovementSystem;

impl<'a> System<'a> for MovementSystem {
    type SystemData = (WriteStorage<'a, Position>, ReadStorage<'a, Velocity>);

    fn run(&mut self, (mut pos, vel): Self::SystemData) {
        for (pos, vel) in (&mut pos, &vel).join() {
            pos.x += vel.x;
            pos.y += vel.y;
        }
    }
}

impl From<&Position> for Vector3 {
    fn from(pos: &Position) -> Vector3 {
        Vector3::new(pos.x, pos.y, 0.0)
    }
}

impl Position {
    pub fn to_vec3(self) -> Vector3 {
        Vector3::new(self.x, self.y, 0.0)
    }
}

impl From<&Position> for Vector2 {
    fn from(pos: &Position) -> Vector2 {
        Vector2::new(pos.x, pos.y)
    }
}

impl From<&Velocity> for Vector2 {
    fn from(pos: &Velocity) -> Vector2 {
        Vector2::new(pos.x, pos.y)
    }
}

pub struct Player {
    entity : Entity,
    speed : f32,
}

impl Player {
    pub fn from_entity(entity: Entity) -> Self { return Self { entity,  speed:  0.05 } }
}

#[derive(Component)]
pub struct Camera {
    pub fov: f32,
    pub up: Vector3,
}

#[derive(Component)]
pub struct Target(pub Entity);

#[derive(Component)]
pub struct Position3D(pub Vector3);

#[derive(Component)]
pub struct ActiveCamera(pub Entity);

#[derive(Component)]
pub struct PlayerCamera(pub Entity);

#[derive(Component)]
pub struct SphericalOffset {
    pub theta: f32,
    pub phi: f32,
    pub radius: f32,
    pub theta_delta: f32,
    pub phi_delta: f32,
    pub radius_delta: f32,
}

// Note(Jökull): Until we have a standardized way of interacting or setting these values,
//               we can have the defaults as the most practical
impl SphericalOffset {
    pub fn new() -> Self { Self {
        theta: PI / 3.0,
        phi: 0.2 * PI,
        radius: 15.0,
        // TODO: Not satisfactory, but need to limit untraceable magic constants
        theta_delta: -0.005,
        phi_delta: 0.005,
        radius_delta: 0.1,
    }}
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
    type SystemData = (WriteStorage<'a, Camera>, ReadStorage<'a, Position3D>);

    fn run(&mut self, (camera, pos): Self::SystemData) {}
}

#[derive(Component)]
pub struct Model3D {
    pub idx: usize,
    offset: Vector3,
    scale: f32,
    z_rotation : f32,
    tint: Color,
}

// Note(Jökull): Probably not great to have both constructor and builder patterns
impl Model3D {
    pub fn new() -> Self { Self { idx: 0, offset: Vector3::zero(), tint: Color::WHITE, scale: 1.0, z_rotation: 0.0} }
    pub fn from_index(index: usize) -> Model3D { let mut m = Self::new(); m.idx = index; return m; }
    pub fn with_offset(mut self, offset: Vector3) -> Model3D { self.offset = offset; self }
    pub fn with_scale(mut self, scale: f32) -> Self { self.scale = scale; self }
    pub fn with_z_rotation(mut self, z_rotation: f32) -> Self { self.z_rotation = z_rotation; self }
    pub fn with_tint(mut self, tint: Color) -> Self { self.tint = tint; self }
}

extern crate raylib;
use raylib::shaders::Shader;

pub struct GraphicsSystem {
    pub thread: RaylibThread,
    pub model_array: Vec<Model>,
    pub l_shader: Shader,
    matModel_loc : i32,
    eyePosition_loc : i32,
}

impl GraphicsSystem {
    pub fn new(thread: RaylibThread, model_array: Vec<Model>, l_shader: Shader) -> Self { Self { thread, model_array, l_shader, matModel_loc: 0, eyePosition_loc: 0 } }
}

impl<'a> System<'a> for GraphicsSystem {
    type SystemData = (
        WriteExpect<'a, RaylibHandle>,
        ReadExpect<'a, ActiveCamera>,
        ReadStorage<'a, Camera>,
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

            self.l_shader.set_shader_value(self.eyePosition_loc, camera_position);

            let mut d3 = d2.begin_mode_3D(
                Camera3D::perspective(
                    camera_position,
                    pos.get(active_target.0).unwrap().to_vec3(),
                    active_camera.up,
                    active_camera.fov,
                )
            );

            for (pos, model) in (&pos, &models).join() {
                let model_pos = pos.clone().to_vec3() + model.offset;

                self.l_shader.set_shader_value_matrix(
                    self.matModel_loc,
                    Matrix::scale(model.scale, model.scale, model.scale).mul(Matrix::translate(model_pos.x, model_pos.y, model_pos.z)),
                );

                d3.draw_model_ex(
                    &self.model_array[model.idx],
                    model_pos,
                    Vector3::new(0.0, 0.0, 1.0),
                    model.z_rotation,
                    Vector3::new(model.scale, model.scale, model.scale),
                    model.tint
                );
            }
        }

        d2.draw_text("deeper", 12, 12, 30, Color::WHITE);
        d2.draw_text(&format!("FPS {}", fps), 12, 46, 18, Color::WHITE);
    }

    fn setup(&mut self, world: &mut World) {
        self.matModel_loc     = self.l_shader.get_shader_location("matModel");
        self.eyePosition_loc  = self.l_shader.get_shader_location("eyePosition");

        println!("GraphicsSystem setup!");
    }
}

pub struct PlayerSystem {
    // Note(Jökull): Yeah, I know. This is just while we're feeling out what is the
    //               responsibility of the input handling system exactly
    last_mouse_pos: Vector2,
}

impl PlayerSystem {
    pub fn new() -> Self { Self { last_mouse_pos: Vector2::zero() } }
}

// Note(Jökull): Is this really just the input handler?
impl<'a> System<'a> for PlayerSystem {
    type SystemData = (
        ReadExpect<'a, RaylibHandle>,
        ReadExpect<'a, Player>,
        ReadExpect<'a, PlayerCamera>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Position3D>,
        ReadStorage<'a, Camera>,

        WriteStorage<'a, Model3D>,
        WriteStorage<'a, Velocity>,
        WriteStorage<'a, SphericalOffset>,
    );

    fn run(&mut self, (rl, player, player_cam, pos, pos3d, cam, mut model, mut vel, mut offset): Self::SystemData) {
        use raylib::consts::{KeyboardKey::*, MouseButton::*};
        let camera = cam.get(player_cam.0).unwrap();
        let camera_pos = pos3d.get(player_cam.0).unwrap();
        let mut camera_offset = offset.get_mut(player_cam.0).unwrap();

        let mouse_delta = rl.get_mouse_position() - self.last_mouse_pos;
        self.last_mouse_pos = rl.get_mouse_position();

        if rl.is_mouse_button_down(MOUSE_MIDDLE_BUTTON) {
            camera_offset.theta += camera_offset.theta_delta * mouse_delta.x;
            camera_offset.phi   += camera_offset.phi_delta   * mouse_delta.y;
            camera_offset.phi = camera_offset.phi.max(0.1 * PI).min(0.25 * PI);
        }

        let mut player_vel = vel.get_mut(player.entity).unwrap();
        player_vel.x = 0.0;
        player_vel.y = 0.0;

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
            let difference = (ray_hit - player_pos.to_vec3()).normalized();
            player_vel.x = difference.x * player.speed;
            player_vel.y = difference.y * player.speed;

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

#[derive(Component)]
struct WallTile;

#[derive(Component)]
struct FloorTile;


use crate::dung_gen::{DungGen};

pub struct DunGenSystem {
    pub dungeon : DungGen,
}

impl<'a> System<'a> for DunGenSystem {
    type SystemData = ();

    fn run(&mut self, (): Self::SystemData) {

    }

    fn setup(&mut self, world: &mut World) {
        use crate::dung_gen::{FLOOR, WALL, WALL_NORTH, WALL_SOUTH, WALL_EAST, WALL_WEST, NOTHING};

        for x in 0..=self.dungeon.width {
            for y in 0..=self.dungeon.height {
                match self.dungeon.world.get(&(x, y)) {
                    None => (),
                    Some(&value) => {
                        match value {
                            FLOOR => {
                                world.create_entity()
                                    .with(Position { x: x as f32, y: y as f32 })
                                    .with(FloorTile)
                                    .with(Model3D::from_index(1).with_tint(Color::DARKGRAY))
                                    .build();
                            },
                            WALL => {
                                world.create_entity()
                                    .with(Position { x: x as f32, y: y as f32 })
                                    .with(WallTile)
                                    .with(Model3D::from_index(0).with_tint(Color::LIGHTGRAY))
                                    .build();
                            },
                            WALL_NORTH => {
                                world.create_entity()
                                    .with(Position { x: x as f32, y: y as f32 })
                                    .with(WallTile)
                                    .with(Model3D::from_index(3)
                                        .with_tint(Color::DARKGRAY)
                                        .with_z_rotation(0.0)
                                    ).build();
                            },
                            WALL_SOUTH => {
                                world.create_entity()
                                    .with(Position { x: x as f32, y: y as f32 })
                                    .with(WallTile)
                                    .with(Model3D::from_index(3)
                                        .with_tint(Color::DARKGRAY)
                                        .with_z_rotation(180.0)
                                    ).build();
                            },
                            WALL_EAST => {
                                world.create_entity()
                                    .with(Position { x: x as f32, y: y as f32 })
                                    .with(WallTile)
                                    .with(Model3D::from_index(3)
                                        .with_tint(Color::DARKGRAY)
                                        .with_z_rotation(-90.0)
                                    ).build();
                            },
                            WALL_WEST => {
                                world.create_entity()
                                    .with(Position { x: x as f32, y: y as f32 })
                                    .with(WallTile)
                                    .with(Model3D::from_index(3)
                                        .with_tint(Color::DARKGRAY)
                                        .with_z_rotation(90.0)
                                    ).build();
                            },
                            // Note(Jökull): Way too slow
                            //NOTHING => {
                            //    world.create_entity()
                            //        .with(Position { x: x as f32, y: y as f32 })
                            //        .with(FloorTile)
                            //        .with(
                            //            Model3D::from_index(1)
                            //                .with_tint(Color::DARKGRAY)
                            //                .with_offset(Vector3::new(0.0, 0.0, 1.0))
                            //        ).build();
                            //},
                            _ => (),
                        }
                    }
                }
            }
        }
    }
}

pub(crate) fn register_components(world: &mut World) {
    world.register::<Position>();
    world.register::<Position3D>();
    world.register::<Velocity>();
    world.register::<Camera>();
    world.register::<Target>();
    world.register::<ActiveCamera>();
    world.register::<SphericalOffset>();
    world.register::<Model3D>();
    world.register::<WallTile>();
    world.register::<FloorTile>();
}