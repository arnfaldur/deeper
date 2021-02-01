use std::collections::HashMap;

use cgmath::{Deg, Vector2, Vector3};
use legion::world::SubWorld;
use legion::*;
use rand::prelude::*;
use wgpu::util::DeviceExt;
use zerocopy::AsBytes;

use crate::components::*;
use crate::dung_gen::DungGen;
use crate::{graphics, loader};
//pub struct MapSwitchingSystem;
//
//impl<'a> System<'a> for MapSwitchingSystem {
//    type SystemData = (
//        ReadExpect<'a, Player>,
//        WriteExpect<'a, MapTransition>,
//        ReadStorage<'a, Position>,
//        ReadStorage<'a, MapSwitcher>,
//    );
//
//    fn run(&mut self, (player, mut trans, pos, switcher): Self::SystemData) {
//        let player_pos = pos.get(player.entity).expect("The player has no position, can't run MapSwitchingSystem");
//        for (pos, switcher) in (&pos, &switcher).join() {
//            if pos.0.distance(player_pos.0) < 0.5 {
//                *trans = switcher.0;
//            }
//        }
//    }
//}
//

#[system]
#[read_component(TileType)]
#[read_component(Faction)]
pub fn dung_gen(
    world: &mut SubWorld,
    commands: &mut legion::systems::CommandBuffer,
    #[resource] trans: &mut MapTransition,
    #[resource] floor: &mut FloorNumber,
    #[resource] context: &graphics::Context,
    #[resource] ass_man: &loader::AssetManager,
    #[resource] player: &Player,
    #[resource] player_camera: &PlayerCamera,
) {
    match *trans {
        MapTransition::Deeper => {
            // TODO(Arnaldur): bruh
            for (entity, _) in <(Entity, &TileType)>::query().iter(world) {
                commands.remove(*entity);
            }

            for chunk in <&Faction>::query().iter_chunks_mut(world) {
                for (entity, faction) in chunk.into_iter_entities() {
                    if let Faction::Enemies = faction {
                        commands.remove(entity);
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
                (x + rng.gen_range(-2..2), y + rng.gen_range(-2..2))
            };

            // Reset player position and stuff
            commands.add_component(
                player.entity,
                Position(Vector2::new(player_start.0 as f32, player_start.1 as f32)),
            );
            //updater.insert(player.entity, Orientation(Deg(0.0)));
            commands.add_component(player.entity, Velocity::new());

            commands.add_component(
                player_camera.entity,
                Position(Vector2::new(player_start.0 as f32, player_start.1 as f32)),
            );

            // TODO: Turn lights into entities
            {
                let mut init_encoder = context
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

                let mut lights: graphics::Lights = Default::default();

                lights.directional_light = graphics::DirectionalLight {
                    direction: [1.0, 0.8, 0.8, 0.0],
                    ambient: [0.01, 0.015, 0.02, 1.0],
                    color: [0.02, 0.025, 0.05, 1.0],
                };

                for (i, &(x, y)) in dungeon.room_centers.iter().enumerate() {
                    if i >= graphics::MAX_NR_OF_POINT_LIGHTS {
                        break;
                    }
                    lights.point_lights[i] = Default::default();
                    lights.point_lights[i].radius = 10.0;
                    lights.point_lights[i].position = [x as f32, y as f32, 5.0, 1.0];
                    lights.point_lights[i].color = [2.0, 1.0, 0.1, 1.0];
                }

                // TODO: go away
                let temp_buf =
                    context
                        .device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: None,
                            contents: lights.as_bytes(),
                            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_SRC,
                        });

                // TODO: you are not welcome here copy_buffer_to_buffer
                init_encoder.copy_buffer_to_buffer(
                    &temp_buf,
                    0,
                    &context.lights_buf,
                    0,
                    std::mem::size_of::<graphics::Lights>() as u64,
                );

                let command_buffer = init_encoder.finish();

                context.queue.submit(std::iter::once(command_buffer));
                // End graphics shit
            }

            let (cube_idx, plane_idx, wall_idx, stairs_down_idx, floor_idx) = {
                (
                    ass_man.get_model_index("cube.obj").unwrap(),
                    ass_man.get_model_index("plane.obj").unwrap(),
                    ass_man.get_model_index("Wall.obj").unwrap(),
                    ass_man.get_model_index("StairsDown.obj").unwrap(),
                    ass_man.get_model_index("floortile.obj").unwrap(),
                )
            };
            let pathability_map: HashMap<(i32, i32), Entity> = HashMap::new();

            for (&(x, y), &tile_type) in dungeon.world.iter() {
                let pos = Vector2::new(x as f32, y as f32);

                let model = StaticModel::new(
                    &context,
                    match tile_type {
                        TileType::Nothing => plane_idx,
                        TileType::Wall(None) => cube_idx,
                        TileType::Wall(Some(_)) => wall_idx,
                        TileType::Floor => floor_idx,
                        TileType::Path => floor_idx,
                        TileType::LadderDown => stairs_down_idx,
                    },
                    pos.extend(match tile_type {
                        TileType::Nothing => 1.,
                        _ => 0.,
                    }),
                    1.0,
                    match tile_type {
                        TileType::Wall(Some(WallDirection::North)) => 0.,
                        TileType::Wall(Some(WallDirection::West)) => 90.,
                        TileType::Wall(Some(WallDirection::South)) => 180.,
                        TileType::Wall(Some(WallDirection::East)) => 270.,
                        _ => 0.,
                    },
                    match tile_type {
                        TileType::Nothing => graphics::Material::dark_stone(),
                        TileType::Path => graphics::Material::glossy(Vector3::new(0.1, 0.1, 0.1)),
                        _ => graphics::Material::darkest_stone(),
                    },
                );
                let entity = commands.push((model, tile_type));
                match tile_type {
                    TileType::Nothing => {}
                    _ => {
                        commands.add_component(entity, Position(pos));
                    }
                }
                // tile specific behaviors
                match tile_type {
                    TileType::Wall(_) => {
                        commands.add_component(entity, StaticBody);
                        commands.add_component(entity, SquareCollider { side_length: 1.0 });
                    }
                    TileType::LadderDown => {
                        commands.add_component(entity, MapSwitcher(MapTransition::Deeper));
                    }
                    _ => {}
                }
                // TODO: use or delete for pathfinding
                //match tile_type {
                //    TileType::Floor | TileType::Path | TileType::LadderDown => {
                //        pathability_map.insert((x, y), entity);
                //    }
                //    _ => {}
                //}
                // Add enemies to floor
                if TileType::Floor == tile_type
                    && rng.gen_bool(((floor.0 - 1) as f64 * 0.05 + 1.).log2().min(1.) as f64)
                {
                    let rad = rng.gen_range(0.1..0.4) + rng.gen_range(0.0..0.1);
                    let enemy = commands.push((
                        Position(
                            pos + Vector2::new(rng.gen_range(-0.3..0.3), rng.gen_range(-0.3..0.3)),
                        ),
                        Speed(rng.gen_range(1.0..4.0) - 1.6 * rad),
                        Acceleration(rng.gen_range(3.0..9.0) + 2.0 * rad),
                        Orientation(Deg(0.0)),
                        Velocity::new(),
                        DynamicBody { mass: rad },
                        CircleCollider { radius: rad },
                    ));
                    commands.add_component(enemy, Faction::Enemies);
                    commands.add_component(
                        enemy,
                        AIFollow {
                            target: player.entity,
                            minimum_distance: 2.0 + rad,
                        },
                    );
                    commands.add_component(
                        enemy,
                        HitPoints {
                            max: rng.gen_range(0.0..2.0) + 8. * rad,
                            health: rng.gen_range(0.0..2.0) + 8. * rad,
                        },
                    );
                    commands.add_component(
                        enemy,
                        Model3D::from_index(
                            &context,
                            ass_man.get_model_index("monstroman.obj").unwrap(),
                        )
                        .with_material(graphics::Material::glossy(Vector3::<f32>::new(
                            rng.gen(),
                            rng.gen(),
                            rng.gen(),
                        )))
                        .with_scale(rad * 1.7),
                    );
                }
            }
            // TODO: use this or delete for a pathfinding system
            // mark pathable neighbours
            //for (&(x, y), &ent) in pathability_map.iter() {
            //    updater.insert(ent, TileNeighbours {
            //        n: pathability_map.get(&(x + 0, y + 1)).cloned(),
            //        w: pathability_map.get(&(x + 1, y + 0)).cloned(),
            //        s: pathability_map.get(&(x + 0, y - 1)).cloned(),
            //        e: pathability_map.get(&(x - 1, y + 0)).cloned(),
            //    });
            //}
        }
        _ => {}
    }
    *trans = MapTransition::None;
}

//pub struct DunGenSystem;
//
//impl<'a> System<'a> for DunGenSystem {
//    type SystemData = (
//        WriteExpect<'a, MapTransition>,
//        ReadExpect<'a, graphics::Context>,
//        ReadExpect<'a, loader::AssetManager>,
//        ReadExpect<'a, Player>,
//        ReadExpect<'a, PlayerCamera>,
//        WriteExpect<'a, FloorNumber>,
//        Entities<'a>,
//        ReadStorage<'a, TileType>,
//        ReadStorage<'a, Faction>,
//        Read<'a, LazyUpdate>,
//    );
//
//    fn run(&mut self, (mut trans, context, ass_man, player, player_camera, mut floor, ents, tile, factions, updater): Self::SystemData) {
//        match *trans {
//            MapTransition::Deeper => {
//                // delete ground tiles
//                for (ent, _) in (&ents, &tile).join() {
//                    ents.delete(ent);
//                }
//                // delete enemies
//                for (ent, faction) in (&ents, &factions).join() {
//                    if let Faction::Enemies = faction {
//                        ents.delete(ent);
//                    }
//                }
//                floor.0 += 1;
//                println!("You have reached floor {}", floor.0);
//                let dungeon = DungGen::new()
//                    .width(60)
//                    .height(60)
//                    .n_rooms(10)
//                    .room_min(5)
//                    .room_range(5)
//                    .generate();
//
//                let mut rng = thread_rng();
//                let player_start = {
//                    let (x, y) = dungeon
//                        .room_centers
//                        .choose(&mut rand::thread_rng()).unwrap()
//                        .clone();
//                    (x + rng.gen_range(-2, 2), y + rng.gen_range(-2, 2))
//                };
//
//
//                // Reset player position and stuff
//                updater.insert(player.entity, Position(Vector2::new(player_start.0 as f32, player_start.1 as f32)));
//                //updater.insert(player.entity, Orientation(Deg(0.0)));
//                updater.insert(player.entity, Velocity::new());
//
//                updater.insert(player_camera.entity, Position(Vector2::new(player_start.0 as f32, player_start.1 as f32)));
//
//                // TODO: Turn lights into entities
//                {
//                    let mut init_encoder = context.device.create_command_encoder(
//                        &wgpu::CommandEncoderDescriptor { label: None }
//                    );
//
//                    let mut lights: graphics::Lights = Default::default();
//
//                    lights.directional_light = graphics::DirectionalLight {
//                        direction: [1.0, 0.8, 0.8, 0.0],
//                        ambient:   [0.01, 0.015, 0.02, 1.0],
//                        color:     [0.02, 0.025, 0.05, 1.0],
//                    };
//
//                    for (i, &(x, y)) in dungeon.room_centers.iter().enumerate() {
//                        if i >= graphics::MAX_NR_OF_POINT_LIGHTS { break; }
//                        lights.point_lights[i] = Default::default();
//                        lights.point_lights[i].radius = 10.0;
//                        lights.point_lights[i].position = [x as f32, y as f32, 5.0, 1.0];
//                        lights.point_lights[i].color = [2.0, 1.0, 0.1, 1.0];
//                    }
//
//                    // TODO: go away
//                    let temp_buf = context.device.create_buffer_with_data(
//                        lights.as_bytes(),
//                        wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_SRC,
//                    );
//
//                    // TODO: you are not welcome here copy_buffer_to_buffer
//                    init_encoder.copy_buffer_to_buffer(
//                        &temp_buf,
//                        0,
//                        &context.lights_buf,
//                        0,
//                        std::mem::size_of::<graphics::Lights>() as u64,
//                    );
//
//                    let command_buffer = init_encoder.finish();
//
//                    context.queue.submit(&[command_buffer]);
//                    // End graphics shit
//                }
//
//                let (cube_idx, plane_idx, wall_idx, stairs_down_idx, floor_idx) = {
//                    (
//                        ass_man.get_model_index("cube.obj").unwrap(),
//                        ass_man.get_model_index("plane.obj").unwrap(),
//                        ass_man.get_model_index("Wall.obj").unwrap(),
//                        ass_man.get_model_index("StairsDown.obj").unwrap(),
//                        ass_man.get_model_index("floortile.obj").unwrap(),
//                    )
//                };
//                let mut pathability_map: HashMap<(i32, i32), Entity> = HashMap::new();
//
//                for (&(x, y), &tile_type) in dungeon.world.iter() {
//                    let pos = Vector2::new(x as f32, y as f32);
//
//                    let entity = ents.create();
//                    let model = StaticModel::new(
//                        &context,
//                        match tile_type {
//                            TileType::Nothing => plane_idx,
//                            TileType::Wall(None) => cube_idx,
//                            TileType::Wall(Some(_)) => wall_idx,
//                            TileType::Floor => floor_idx,
//                            TileType::Path => floor_idx,
//                            TileType::LadderDown => stairs_down_idx,
//                        },
//                        pos.extend(match tile_type {
//                            TileType::Nothing => 1.,
//                            _ => 0.,
//                        }),
//                        1.0,
//                        match tile_type {
//                            TileType::Wall(Some(WallDirection::North)) => 0.,
//                            TileType::Wall(Some(WallDirection::West)) => 90.,
//                            TileType::Wall(Some(WallDirection::South)) => 180.,
//                            TileType::Wall(Some(WallDirection::East)) => 270.,
//                            _ => 0.,
//                        },
//                        match tile_type {
//                            TileType::Nothing => graphics::Material::dark_stone(),
//                            TileType::Path => graphics::Material::glossy(Vector3::new(0.1, 0.1, 0.1)),
//                            _ => graphics::Material::darkest_stone(),
//                        },
//                    );
//                    updater.insert(entity, model);
//                    updater.insert(entity, tile_type);
//                    match tile_type {
//                        TileType::Nothing => {}
//                        _ => {
//                            updater.insert(entity, Position(pos));
//                        }
//                    }
//                    // tile specific behaviors
//                    match tile_type {
//                        TileType::Wall(_) => {
//                            updater.insert(entity, StaticBody);
//                            updater.insert(entity, SquareCollider { side_length: 1.0 });
//                        }
//                        TileType::LadderDown => {
//                            updater.insert(entity, MapSwitcher(MapTransition::Deeper));
//                        }
//                        _ => {}
//                    }
//                    // TODO: use or delete for pathfinding
//                    //match tile_type {
//                    //    TileType::Floor | TileType::Path | TileType::LadderDown => {
//                    //        pathability_map.insert((x, y), entity);
//                    //    }
//                    //    _ => {}
//                    //}
//                    // Add enemies to floor
//                    if TileType::Floor == tile_type && rng.gen_bool(((floor.0 - 1) as f64 * 0.05 + 1.).log2().min(1.) as f64) {
//                        let rad = rng.gen_range(0.1, 0.4) + rng.gen_range(0.0, 0.1);
//                        let enemy = ents.create();
//                        updater.insert(enemy, Position(pos + Vector2::new(rng.gen_range(-0.3, 0.3), rng.gen_range(-0.3, 0.3))));
//                        updater.insert(enemy, Speed(rng.gen_range(1., 4.) - 1.6 * rad));
//                        updater.insert(enemy, Acceleration(rng.gen_range(3., 9.) + 2.0 * rad));
//                        updater.insert(enemy, Orientation(Deg(0.0)));
//                        updater.insert(enemy, Velocity::new());
//                        updater.insert(enemy, DynamicBody(rad));
//                        updater.insert(enemy, CircleCollider { radius: rad });
//                        updater.insert(enemy, AIFollow {
//                            target: player.entity,
//                            minimum_distance: 2.0 + rad,
//                        });
//                        updater.insert(enemy, Faction::Enemies);
//                        updater.insert(enemy, HitPoints {
//                            max: rng.gen_range(0., 2.) + 8. * rad,
//                            health: rng.gen_range(0., 2.) + 8. * rad,
//                        });
//                        updater.insert(enemy,
//                                       Model3D::from_index(&context, ass_man.get_model_index("monstroman.obj").unwrap())
//                                           .with_material(graphics::Material::glossy(Vector3::<f32>::new(rng.gen(), rng.gen(), rng.gen())))
//                                           .with_scale(rad * 1.7));
//                    }
//                };
//                // TODO: use this or delete for a pathfinding system
//                // mark pathable neighbours
//                //for (&(x, y), &ent) in pathability_map.iter() {
//                //    updater.insert(ent, TileNeighbours {
//                //        n: pathability_map.get(&(x + 0, y + 1)).cloned(),
//                //        w: pathability_map.get(&(x + 1, y + 0)).cloned(),
//                //        s: pathability_map.get(&(x + 0, y - 1)).cloned(),
//                //        e: pathability_map.get(&(x - 1, y + 0)).cloned(),
//                //    });
//                //}
//            }
//            _ => {}
//        }
//        *trans = MapTransition::None;
//    }
//}
//
