use std::collections::HashMap;

use assman::components::{DynamicModelRequest, StaticModelRequest};
use cgmath::{vec2, Vector2};
use entity_smith::Smith;
use graphics::data::LocalUniforms;
use legion::systems::{CommandBuffer, Runnable};
use legion::world::SubWorld;
use legion::{Entity, IntoQuery, SystemBuilder};
use physics::PhysicsEntitySmith;
use rand::prelude::*;
use transforms::{Scale, TransformEntitySmith};

use crate::components::{HitPoints, Player};
use crate::world_gen::components::{
    Direction, Faction, FloorNumber, MapSwitcher, MapTransition, TileType,
};

pub fn dung_gen_system() -> impl Runnable {
    SystemBuilder::new("DungGen System")
        .read_component::<TileType>()
        .read_component::<Faction>()
        .write_resource::<MapTransition>()
        .write_resource::<FloorNumber>()
        .read_resource::<Player>()
        .build(move |command_buffer, world, resources, _| {
            dung_gen(
                command_buffer,
                world,
                &mut resources.0,
                &mut resources.1,
                &resources.2,
            );
        })
}

pub fn dung_gen(
    command_buffer: &mut legion::systems::CommandBuffer,
    world: &mut SubWorld,
    transition: &mut MapTransition,
    floor: &mut FloorNumber,
    player: &Player,
) {
    #[allow(clippy::single_match)]
    match *transition {
        MapTransition::Deeper => {
            // TODO(Arnaldur): bruh
            for (entity, _) in <(Entity, &TileType)>::query().iter(world) {
                command_buffer.remove(*entity);
            }

            for (entity, faction) in <(Entity, &Faction)>::query().iter(world) {
                if let Faction::Enemies = faction {
                    command_buffer.remove(*entity);
                }
            }

            floor.0 += 1;

            println!("You have reached floor {}", floor.0);

            //let mut rng = thread_rng();

            //let wfc_source = image::open("assets/Images/dungeon_5_separated.png").unwrap();
            let wfc_source = image::open("maps/WFC.png").unwrap();

            // use crate::world_gen::grid::Grid;
            // use crate::world_gen::wfc::{wfc, SQUARE_NEIGHBOURHOOD};
            // let wfc_result = wfc(
            //     Grid::from(&wfc_source.into_rgb8()),
            //     &SQUARE_NEIGHBOURHOOD,
            //     Vector2::<usize>::from_value(64),
            //     &[],
            // );

            // let wfc_result = image_patterns
            //     .collapse_wave_retrying(
            //         map_size,
            //         wrap::WrapXY,
            //         EmptyEdgesForbid {
            //             empty_tile_id: top_left_corner_id,
            //         },
            //         retry::Forever,
            //         &mut StdRng::from_entropy(),
            //     )
            //     .apply(|wave| image_patterns.image_from_wave(&wave));
            let wfc_result = wfc_source;

            let test_world = wfc_result
                .into_bgr8()
                .enumerate_pixels()
                .map(|(x, y, pixel)| {
                    ((x as i32, y as i32), {
                        let [b, g, r] = pixel.0;
                        let direction = match r {
                            0 => Direction::North,
                            64 => Direction::East,
                            128 => Direction::South,
                            192 => Direction::West,
                            _ => Direction::North,
                        };
                        if b > 0 {
                            match g {
                                0 => TileType::Floor,
                                64 => TileType::CornerIn(direction),
                                128 => TileType::CornerOut(direction),
                                192 => TileType::Wall(direction),
                                _ => TileType::Unknown,
                            }
                        } else {
                            TileType::Nothing
                        }
                    })
                })
                .collect::<HashMap<(i32, i32), TileType>>();

            populate_environment(command_buffer, &test_world);

            let player_start = test_world
                .iter()
                .filter(|&(_, &tile_type)| tile_type == TileType::Floor)
                .nth(100)
                //.choose(&mut rng)
                .map(|((x, y), _)| vec2(*x as f32, *y as f32))
                .unwrap();

            // Reset player position and stuff
            command_buffer
                .forge(player.player)
                .position(player_start.extend(0.))
                .velocity_zero();

            add_enemies(command_buffer, floor, &test_world);
        }
        _ => {}
    }
    *transition = MapTransition::None;
}

fn populate_environment(
    command_buffer: &mut CommandBuffer,
    dungeon: &HashMap<(i32, i32), TileType>,
) {
    for (&(x, y), &tile_type) in dungeon.iter() {
        let pos = Vector2::new(x as f32, y as f32);

        let mut smith = command_buffer.smith();

        let thing = StaticModelRequest::new(
            match tile_type {
                TileType::Nothing => "DevFloor.obj",
                TileType::Wall(_) => "DevWall.obj",
                TileType::Floor => "DevFloor.obj",
                TileType::Path => "DevFloor.obj",
                TileType::CornerIn(_) => "DevCornerIn.obj",
                TileType::CornerOut(_) => "DevCornerOut.obj",
                TileType::LadderDown => "DevFloor.obj",

                TileType::Unknown => "cube.obj",
                TileType::UndirectedWall => "cube.obj",
            },
            LocalUniforms::simple(
                pos.extend(match tile_type {
                    TileType::Nothing => 1.50,
                    _ => 0.,
                })
                .into(),
                1.0,
                match tile_type {
                    TileType::Wall(Direction::North) => 0.,
                    TileType::Wall(Direction::West) => 90.,
                    TileType::Wall(Direction::South) => 180.,
                    TileType::Wall(Direction::East) => 270.,

                    // TODO: Fix orientation of model s.t. North = 0.0
                    TileType::CornerIn(Direction::North) => 270.,
                    TileType::CornerIn(Direction::West) => 0.,
                    TileType::CornerIn(Direction::South) => 90.,
                    TileType::CornerIn(Direction::East) => 180.,

                    TileType::CornerOut(Direction::North) => 270.,
                    TileType::CornerOut(Direction::West) => 0.,
                    TileType::CornerOut(Direction::South) => 90.,
                    TileType::CornerOut(Direction::East) => 180.,

                    _ => 0.,
                },
                Default::default(),
            ),
        );

        smith.any(thing);

        smith.any(tile_type);
        match tile_type {
            TileType::Nothing => {}
            _ => {
                smith.pos(pos);
            }
        }

        // tile specific behaviors
        match tile_type {
            TileType::Wall(_) | TileType::CornerIn(_) | TileType::CornerOut(_) => {
                smith.static_square_body(1.0);
            }
            TileType::LadderDown => {
                smith.any(MapSwitcher(MapTransition::Deeper));
            }
            _ => {}
        }
    }
}

fn add_enemies(
    command_buffer: &mut CommandBuffer,
    floor: &mut FloorNumber,
    dungeon: &HashMap<(i32, i32), TileType>,
) {
    let mut rng = thread_rng();

    // Add enemies to floor

    for (&(x, y), &tile_type) in dungeon.iter() {
        let pos = Vector2::new(x as f32, y as f32);

        if TileType::Floor == tile_type
            && rng.gen_bool(((floor.0 - 1) as f64 * 0.05 + 1.).log2().min(1.) as f64)
        {
            let rad = rng.gen_range(0.1..0.4) + rng.gen_range(0.0..0.1);
            let mut smith = command_buffer.smith();
            smith
                .position(
                    (pos + Vector2::new(rng.gen_range(-0.3..0.3), rng.gen_range(-0.3..0.3)))
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
                .any(HitPoints {
                    max: rng.gen_range(0.0..2.0) + 8. * rad,
                    health: rng.gen_range(0.0..2.0) + 8. * rad,
                })
                .any(DynamicModelRequest {
                    label: "monstroman.obj".to_string(),
                })
                .any(Scale(rad * 1.7))
                .done();
        }
    }
}
