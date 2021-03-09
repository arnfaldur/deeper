use cgmath::{vec2, SquareMatrix, Vector2, Vector3};
use legion::world::SubWorld;
use legion::*;
use rand::prelude::*;

use crate::components::entity_builder::EntitySmith;
use crate::components::*;
use crate::dung_gen::DungGen;
use crate::transform::components::Position;
use crate::{assets, graphics};

#[system]
#[read_component(TileType)]
#[read_component(Faction)]
pub fn dung_gen(
    world: &mut SubWorld,
    commands: &mut legion::systems::CommandBuffer,
    #[resource] trans: &mut MapTransition,
    #[resource] floor: &mut FloorNumber,
    #[resource] context: &mut graphics::Context,
    #[resource] ass_man: &mut assets::AssetManager,
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

            let mut optimizer = StaticMeshOptimizer::new();

            for (&(x, y), &tile_type) in dungeon.world.iter() {
                let pos = Vector2::new(x as f32, y as f32);

                optimizer.insert(
                    match tile_type {
                        TileType::Nothing => plane_idx,
                        TileType::Wall(None) => cube_idx,
                        TileType::Wall(Some(_)) => wall_idx,
                        TileType::Floor => floor_idx,
                        TileType::Path => floor_idx,
                        TileType::LadderDown => stairs_down_idx,
                    },
                    LocalUniforms::simple(
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
                );

                let entity = commands.push((tile_type,));
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

            let optimized_models = optimizer
                .finish(ass_man)
                .iter()
                .map(|(uniforms, vertex_lists)| {
                    (
                        *uniforms,
                        ass_man.allocate_graphics_model_from_vertex_lists(
                            context,
                            vertex_lists.clone(),
                        ),
                    )
                })
                .collect_vec();

            for (uniforms, idx) in optimized_models.iter() {
                commands.push((StaticModel::from_uniforms(context, *idx, *uniforms),));
            }
        }
        _ => {}
    }
    *trans = MapTransition::None;
}

// TODO: Find a place for this

use graphics::data::LocalUniforms;
use itertools::Itertools;

struct StaticMeshOptimizationEntry {
    archetype: LocalUniforms,
    instances: Vec<(usize, LocalUniforms)>,
}

struct StaticMeshOptimizer {
    entries: Vec<StaticMeshOptimizationEntry>,
}

impl StaticMeshOptimizer {
    fn new() -> Self { Self { entries: vec![] } }

    fn insert(&mut self, idx: usize, local_uniforms: LocalUniforms) {
        for entry in self.entries.iter_mut() {
            if entry.archetype.similar_to(&local_uniforms) {
                entry.instances.push((idx, local_uniforms));
                return;
            }
        }
        self.entries.push(StaticMeshOptimizationEntry {
            archetype: local_uniforms.with_model_matrix(cgmath::Matrix4::identity().into()),
            instances: vec![(idx, local_uniforms)],
        })
    }

    fn finish(
        &self,
        ass_man: &assets::AssetManager,
    ) -> Vec<(LocalUniforms, graphics::data::VertexLists)> {
        self.entries
            .iter()
            .map(|entry| {
                (
                    entry.archetype,
                    entry
                        .instances
                        .iter()
                        .map(|(idx, uniforms)| {
                            ass_man
                                .models
                                .get(*idx)
                                .unwrap()
                                .vertex_lists
                                .iter()
                                .map(|vertex_list| {
                                    vertex_list
                                        .iter()
                                        .map(|vertex| vertex.transformed(uniforms.model_matrix))
                                        .collect_vec()
                                })
                                .collect_vec()
                        })
                        .collect_vec()
                        .concat(),
                )
            })
            .collect_vec()
    }
}
