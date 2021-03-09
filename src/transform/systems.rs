use cgmath::Matrix4;
use imgui::Ui;
use legion::systems::{Builder, ParallelRunnable, Runnable};
use legion::{component, Entity, EntityStore, IntoQuery, SystemBuilder};

use crate::components::{Children, Name, Parent};
use crate::graphics::gui::GuiContext;
use crate::transform::*;

pub(crate) trait TransformBuilderExtender {
    //fn add_transform_systems(&mut self, resources: &mut Resources) -> &mut Self;
    fn add_transform_systems(&mut self) -> &mut Self;
}

impl TransformBuilderExtender for Builder {
    //fn add_transform_systems(&mut self, resources: &mut Resources) -> &mut Self {
    fn add_transform_systems(&mut self) -> &mut Self {
        self.add_system(populate_transforms())
            .add_system(depopulate_transforms())
            .add_system(adopt_children())
            .flush()
            .add_system(validate_rotation_transforms())
            .add_system(reset_transforms())
            .add_system(position())
            .add_system(rotation())
            .add_system(rotation3d())
            .add_system(scale())
            .add_system(inherit_transforms()) // FIXME: this should not have to be thread local but the scheduling is weird so that seems to be needed
                                              //.add_thread_local(player_transform_shower())
    }
}

#[allow(dead_code)]
fn player_transform_shower() -> impl Runnable {
    SystemBuilder::new("player_transform_shower")
        .read_component::<Transform>()
        .with_query(<&Transform>::query())
        .read_resource::<crate::components::Player>()
        .read_resource::<crate::components::PlayerCamera>()
        .write_resource::<GuiContext>()
        .build(move |_cmd, world, resources, queries| {
            let (player, player_camera, _) = resources;

            fn display_table(ui: &Ui, table: [[f32; 4]; 4]) {
                ui.columns(4, im_str!("a table"), true);
                for i in 0..4 {
                    ui.separator();
                    for col in table.iter() {
                        ui.text(format!("{:.2}", col[i]));
                        ui.next_column();
                    }
                }
                ui.columns(1, im_str!("this is dumb"), false);
                ui.separator();
            }

            use imgui::{im_str, Condition};
            let names = [
                im_str!("player"),
                im_str!("player model"),
                im_str!("camera"),
            ];

            for i in 0..3 {
                let entity = [player.player, player.model, player_camera.entity][i];
                GuiContext::with_ui(|ui| {
                    imgui::Window::new(names[i])
                        .position([400.0 + i as f32 * 200.0, 50.0], Condition::FirstUseEver)
                        .size([200.0, 280.0], Condition::FirstUseEver)
                        .build(ui, || {
                            if let Ok(trans) = queries.get(world, entity) {
                                ui.text("Absolute transform:");
                                display_table(ui, trans.absolute.into());
                                ui.text("Relative transform:");
                                display_table(ui, trans.relative.into());
                            }
                        });
                });
            }
        })
}

fn populate_transforms() -> impl Runnable {
    SystemBuilder::new("populate_transforms")
        .with_query(<Entity>::query().filter(
            !component::<Transform>()
                & (component::<Position>()
                    | component::<Rotation>()
                    | component::<Rotation3D>()
                    | component::<Scale>()),
        ))
        .build(move |cmd, world, _, query| {
            query.for_each_mut(world, |ent: &Entity| {
                cmd.add_component(*ent, Transform::identity());
            });
        })
}

fn depopulate_transforms() -> impl Runnable {
    SystemBuilder::new("depopulate_transforms")
        .with_query(<Entity>::query().filter(
            component::<Transform>()
                & !component::<Position>()
                & !component::<Rotation>()
                & !component::<Rotation3D>()
                & !component::<Scale>(),
        ))
        .build(move |cmd, world, _, query| {
            query.for_each_mut(world, |ent: &Entity| {
                cmd.remove_component::<Transform>(*ent);
            });
        })
}

fn adopt_children() -> impl Runnable {
    SystemBuilder::new("adopt_children")
        .with_query(<(Entity, &Parent)>::query())
        .write_component::<Children>()
        .build(move |cmd, world, _, query| {
            let (child_world, mut parent_world) = world.split_for_query(query);
            query.for_each(&child_world, |(ent, par): (&Entity, &Parent)| {
                if let Ok(entry) = parent_world.entry_mut(par.0) {
                    // FIXME: The entitysmith should probably handle the actual adoption
                    entry.into_component_mut::<Children>().map_or_else(
                        |_| {
                            cmd.add_component(par.0, Children(vec![*ent].into_iter().collect()));
                        },
                        |children| {
                            children.0.insert(*ent);
                        },
                    )
                }
            });
        })
}

// when a transform informing component changes, reset the transform such that it can be recalculated
fn reset_transforms() -> impl Runnable {
    SystemBuilder::new("populate_transforms")
        .with_query(
            <&mut Transform>::query(),
            //     .filter( maybe_changed::<Position>()
            //         | maybe_changed::<Position3D>()
            //         | maybe_changed::<Rotation>()
            //         | maybe_changed::<Rotation3D>()
            //         | maybe_changed::<Scale>()
            //         | maybe_changed::<NonUniformScale>(),
            // )
        )
        .build(move |_, world, _, query| {
            query.for_each_mut(world, |transform: &mut Transform| {
                *transform = Transform::identity();
            });
        })
}

fn validate_rotation_transforms() -> impl Runnable {
    SystemBuilder::new("validate_rotation_transforms")
        .with_query(<(Option<&Name>, &Rotation, &Rotation3D)>::query())
        .build(move |_, world, _, query| {
            query.for_each(world, |(name, _, _)| {
                eprint!("Note: ");
                if let Some(name) = name {
                    eprint!("The entity {} ", name);
                } else {
                    eprint!("an entity ");
                }
                eprintln!("has both Rotation and Rotation3D components");
            })
        })
}

#[allow(dead_code)]
fn calculate_relative_transforms() -> impl ParallelRunnable {
    SystemBuilder::new("transforms_position")
        .write_component::<Transform>()
        .read_component::<Position>()
        .read_component::<Rotation>()
        .read_component::<Rotation3D>()
        .read_component::<Scale>()
        .with_query(<(&mut Transform, &Position)>::query())
        .with_query(<(&mut Transform, &Rotation)>::query())
        .with_query(<(&mut Transform, &Rotation3D)>::query())
        .with_query(<(&mut Transform, &Scale)>::query())
        .build(move |_, world, _, query| {
            query.0.for_each_mut(world, |(transform, val)| {
                transform.relative = transform.relative * Matrix4::from(val);
            });
            query.1.for_each_mut(world, |(transform, val)| {
                transform.relative = transform.relative * Matrix4::from(val);
            });
            query.2.for_each_mut(world, |(transform, val)| {
                transform.relative = transform.relative * Matrix4::from(val);
            });
            query.3.for_each_mut(world, |(transform, val)| {
                transform.relative = transform.relative * Matrix4::from(val);
            });
        })
}

fn position() -> impl ParallelRunnable {
    SystemBuilder::new("transforms_position")
        .write_component::<Transform>()
        .with_query(<(&mut Transform, &Position)>::query())
        .build(move |_, world, _, query| {
            query.for_each_mut(world, |(transform, val)| {
                transform.relative = transform.relative * Matrix4::from(val);
            });
        })
}
fn rotation() -> impl ParallelRunnable {
    SystemBuilder::new("transforms_rotation")
        .write_component::<Transform>()
        .with_query(<(&mut Transform, &Rotation)>::query())
        .build(move |_, world, _, query| {
            query.for_each_mut(world, |(transform, val)| {
                transform.relative = transform.relative * Matrix4::from(val);
            });
        })
}
fn rotation3d() -> impl ParallelRunnable {
    SystemBuilder::new("transforms_rotation3d")
        .write_component::<Transform>()
        .with_query(<(&mut Transform, &Rotation3D)>::query())
        .build(move |_, world, _, query| {
            query.for_each_mut(world, |(transform, val)| {
                transform.relative = transform.relative * Matrix4::from(val);
            });
        })
}

fn scale() -> impl ParallelRunnable {
    SystemBuilder::new("transforms_scale")
        .write_component::<Transform>()
        .with_query(<(&mut Transform, &Scale)>::query())
        .build(move |_, world, _, query| {
            query.for_each_mut(world, |(transform, val)| {
                transform.relative = transform.relative * Matrix4::from(val);
            });
        })
}

// fn non_uniform_scale() -> impl ParallelRunnable {
//     SystemBuilder::new("transforms_non_uniform_scale")
//         .write_component::<Transform>()
//         .with_query(<(&mut Transform, &NonUniformScale)>::query())
//         .build(move |_, world, _, query| {
//             query.for_each_mut(world, |(transform, val)| {
//                 transform.relative = transform.relative * Matrix4::from(val);
//             });
//         })
// }

fn inherit_transforms() -> impl ParallelRunnable {
    SystemBuilder::new("inherit_transforms")
        .with_query(<&mut Transform>::query().filter(!component::<Parent>()))
        .with_query(
            <(Entity, &Transform)>::query()
                .filter(!component::<Parent>() & component::<Children>()),
        )
        .read_component::<Children>()
        .write_component::<Transform>()
        .build(move |_, world, _, (first_generation, parents)| {
            first_generation.for_each_mut(world, |transform: &mut Transform| {
                transform.absolute = transform.relative;
            });
            // add all parents to a list
            let mut stack = Vec::new();
            parents.for_each_mut(world, |(entity, transform): (&Entity, &Transform)| {
                stack.push((*entity, *transform));
            });

            // apply the transforms through breadth first traversal
            let (children_only, mut rest) = world.split::<&Children>();
            while let Some((parent, parent_transform)) = stack.pop() {
                if let Ok(children) = <&Children>::query().get(&children_only, parent) {
                    for &child in &children.0 {
                        if let Ok(child_transform) =
                            <&mut Transform>::query().get_mut(&mut rest, child)
                        {
                            child_transform.absolute =
                                parent_transform.absolute * child_transform.relative;
                            stack.push((child, *child_transform));
                        }
                    }
                }
            }
        })
}
