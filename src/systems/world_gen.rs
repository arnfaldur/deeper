use std::collections::HashMap;

use cgmath::{vec2, Vector2, Vector3};
use legion::world::SubWorld;
use legion::*;
use rand::prelude::*;

use crate::components::entity_builder::EntitySmith;
use crate::components::*;
use crate::dung_gen::DungGen;
use crate::transform::components::Position;
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
                vec2(
                    (x + rng.gen_range(-2..2)) as f32,
                    (y + rng.gen_range(-2..2)) as f32,
                )
            };

            // Reset player position and stuff
            EntitySmith::from_entity(commands, player.player)
                .position(player_start.extend(0.))
                .velocity_zero();
            // commands.add_component(player.player, Position(player_start.extend(0.)));
            // commands.add_component(player.player, Velocity::new());

            // TODO: Turn lights into entities
            {
                let mut lights: graphics::data::Lights = Default::default();

                lights.directional_light = graphics::data::DirectionalLight {
                    direction: [1.0, 0.8, 0.8, 0.0],
                    ambient: [0.01, 0.015, 0.02, 1.0],
                    color: [0.02, 0.025, 0.05, 1.0],
                };

                for (i, &(x, y)) in dungeon.room_centers.iter().enumerate() {
                    if i >= graphics::MAX_NR_OF_POINT_LIGHTS {
                        break;
                    }
                    lights.point_lights[i] = graphics::data::PointLight {
                        position: [x as f32, y as f32, 5.0, 1.0],
                        radius: 10.0,
                        color: [2.0, 1.0, 0.1, 1.0],
                        ..Default::default()
                    };
                }

                context.queue.write_buffer(
                    &context.model_render_ctx.lights_uniform_buf,
                    0,
                    bytemuck::bytes_of(&lights),
                );

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
            let _pathability_map: HashMap<(i32, i32), Entity> = HashMap::new();

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
                        TileType::Nothing => graphics::data::Material::dark_stone(),
                        TileType::Path => {
                            graphics::data::Material::glossy(Vector3::new(0.1, 0.1, 0.1))
                        }
                        _ => graphics::data::Material::darkest_stone(),
                    },
                );
                let entity = commands.push((model, tile_type));
                match tile_type {
                    TileType::Nothing => {}
                    _ => {
                        commands.add_component(entity, Position(pos.extend(0.)));
                    }
                }
                // tile specific behaviors
                match tile_type {
                    TileType::Wall(_) => {
                        commands.add_component(entity, PhysicsBody::Static);
                        commands.add_component(entity, Collider::Square { side_length: 1.0 });
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
                    let mut smith = EntitySmith::from_buffer(commands);
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
                        .any(
                            Model3D::from_index(ass_man.get_model_index("monstroman.obj").unwrap())
                                .with_material(graphics::data::Material::glossy(
                                    Vector3::<f32>::new(rng.gen(), rng.gen(), rng.gen()),
                                ))
                                .with_scale(rad * 1.7),
                        )
                        .done();
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
