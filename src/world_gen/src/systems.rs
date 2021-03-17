use cgmath::{vec2, Vector2, Vector3};
use components::*;
use entity_smith::Smith;
use graphics;
use graphics::data::LocalUniforms;
use legion::systems::Runnable;
use legion::world::SubWorld;
use legion::{Entity, IntoQuery, SystemBuilder};
use physics::PhysicsEntitySmith;
use rand::prelude::*;
use transforms::TransformEntitySmith;

use crate::components::{DynamicModelRequest, StaticModelRequest, TileType, WallDirection};
use crate::dung_gen::DungGen;

pub fn dung_gen_system() -> impl Runnable {
    SystemBuilder::new("DungGen System")
        .read_component::<TileType>()
        .read_component::<Faction>()
        .write_resource::<MapTransition>()
        .write_resource::<FloorNumber>()
        .write_resource::<graphics::GraphicsContext>()
        .read_resource::<Player>()
        .build(move |command_buffer, world, resources, _| {
            dung_gen(
                command_buffer,
                world,
                &mut resources.0,
                &mut resources.1,
                &mut resources.2,
                &resources.3,
            );
        })
}

pub fn dung_gen(
    command_buffer: &mut legion::systems::CommandBuffer,
    world: &mut SubWorld,
    transition: &mut MapTransition,
    floor: &mut FloorNumber,
    context: &mut graphics::GraphicsContext,
    player: &Player,
) {
    match *transition {
        MapTransition::Deeper => {
            // TODO(Arnaldur): bruh
            for (entity, _) in <(Entity, &TileType)>::query().iter(world) {
                command_buffer.remove(*entity);
            }

            for chunk in <&Faction>::query().iter_chunks_mut(world) {
                for (entity, faction) in chunk.into_iter_entities() {
                    if let Faction::Enemies = faction {
                        command_buffer.remove(entity);
                    }
                }
            }

            floor.0 += 1;

            println!("You have reached floor {}", floor.0);
            let dungeon = DungGen::new()
                .width(60)
                .height(60)
                .n_rooms(10)
                .room_min(5)
                .room_range(5)
                .generate();

            let mut rng = thread_rng();
            let player_start = {
                let (x, y) = dungeon
                    .room_centers
                    .choose(&mut rand::thread_rng())
                    .unwrap()
                    .clone();
                vec2(
                    (x + rng.gen_range(-2..2)) as f32,
                    (y + rng.gen_range(-2..2)) as f32,
                )
            };

            // Reset player position and stuff
            command_buffer
                .forge(player.player)
                .position(player_start.extend(0.))
                .velocity_zero();

            let room_centers = &dungeon.room_centers;
            // TODO: Turn lights into entities
            graphics::util::make_lights(&context, room_centers);

            for (&(x, y), &tile_type) in dungeon.world.iter() {
                let pos = Vector2::new(x as f32, y as f32);

                let mut smith = command_buffer.smith();

                smith.any(StaticModelRequest {
                    label: match tile_type {
                        TileType::Nothing => "plane.obj",
                        TileType::Wall(None) => "cube.obj",
                        TileType::Wall(Some(_)) => "Wall.obj",
                        TileType::Floor => "floortile.obj",
                        TileType::Path => "floortile.obj",
                        TileType::LadderDown => "StairsDown.obj",
                    }
                    .to_string(),
                    uniforms: LocalUniforms::simple(
                        pos.extend(match tile_type {
                            TileType::Nothing => 1.,
                            _ => 0.,
                        })
                        .into(),
                        1.0,
                        match tile_type {
                            TileType::Wall(Some(WallDirection::North)) => 0.,
                            TileType::Wall(Some(WallDirection::West)) => 90.,
                            TileType::Wall(Some(WallDirection::South)) => 180.,
                            TileType::Wall(Some(WallDirection::East)) => 270.,
                            _ => 0.,
                        },
                        match tile_type {
                            TileType::Nothing => graphics::data::Material::dark_stone(),
                            TileType::Path => {
                                graphics::data::Material::glossy(Vector3::new(0.1, 0.1, 0.1))
                            }
                            _ => graphics::data::Material::darkest_stone(),
                        },
                    ),
                });

                smith.any(tile_type);
                match tile_type {
                    TileType::Nothing => {}
                    _ => {
                        smith.pos(pos);
                    }
                }

                // tile specific behaviors
                match tile_type {
                    TileType::Wall(_) => {
                        smith.static_square_body(1.0);
                    }
                    TileType::LadderDown => {
                        smith.any(MapSwitcher(MapTransition::Deeper));
                    }
                    _ => {}
                }

                // Add enemies to floor
                if TileType::Floor == tile_type
                    && rng.gen_bool(((floor.0 - 1) as f64 * 0.05 + 1.).log2().min(1.) as f64)
                {
                    let rad = rng.gen_range(0.1..0.4) + rng.gen_range(0.0..0.1);
                    let mut smith = command_buffer.smith();
                    smith
                        .position(
                            (pos + Vector2::new(
                                rng.gen_range(-0.3..0.3),
                                rng.gen_range(-0.3..0.3),
                            ))
                            .extend(0.),
                        )
                        .agent(
                            rng.gen_range(1.0..4.0) - 1.6 * rad,
                            rng.gen_range(3.0..9.0) + 2.0 * rad,
                        )
                        .orientation(0.0)
                        .velocity_zero()
                        .dynamic_body(rad)
                        .circle_collider(rad)
                        .any(Faction::Enemies)
                        .any(AIFollow {
                            target: player.player,
                            minimum_distance: 2.0 + rad,
                        })
                        .any(HitPoints {
                            max: rng.gen_range(0.0..2.0) + 8. * rad,
                            health: rng.gen_range(0.0..2.0) + 8. * rad,
                        })
                        .any(DynamicModelRequest {
                            label: "monstroman.obj".to_string(),
                            material: graphics::data::Material::glossy(Vector3::<f32>::new(
                                rng.gen(),
                                rng.gen(),
                                rng.gen(),
                            )),
                            scale: rad * 1.7,
                        })
                        .done();
                }
            }
        }
        _ => {}
    }
    *transition = MapTransition::None;
}
