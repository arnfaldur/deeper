use cgmath::Matrix4;
use entity_smith::Smith;
//use imgui::Ui;
use legion::systems::{Builder, ParallelRunnable, Runnable};
use legion::world::EntityAccessError;
use legion::{component, maybe_changed, Entity, IntoQuery, SystemBuilder};

use crate::components::{Children, Parent};
use crate::{Position, Rotation, Scale, SphericalOffset, Transform, TransformEntitySmith};
//use crate::graphics::gui::GuiContext;

// note(Jökull): This belongs here if anywhere
pub fn spherical_offset_system() -> impl ParallelRunnable {
    SystemBuilder::new("spherical_offset")
        .with_query(<(&mut Position, &SphericalOffset)>::query())
        .build(move |_cmd, world, _resources, query| {
            let for_query = world;
            query.for_each_mut(for_query, |components| {
                spherical_offset(components.0, components.1);
            });
        })
}

#[allow(dead_code)]
pub fn spherical_offset(pos: &mut Position, follow: &SphericalOffset) {
    pos.0.x = follow.radius * follow.theta.cos() * follow.phi.cos();
    pos.0.y = follow.radius * follow.theta.sin() * follow.phi.cos();
    pos.0.z = follow.radius * follow.phi.sin();
}

pub trait TransformBuilderExtender {
    //fn add_transform_systems(&mut self, resources: &mut Resources) -> &mut Self;
    fn add_transform_systems(&mut self) -> &mut Self;
}

impl TransformBuilderExtender for Builder {
    //fn add_transform_systems(&mut self, resources: &mut Resources) -> &mut Self {
    fn add_transform_systems(&mut self) -> &mut Self {
        self.add_system(spherical_offset_system())
            .add_system(populate_transforms())
            .add_system(depopulate_transforms())
            .add_system(adopt_children())
            .flush()
            .add_system(reset_transforms())
            .add_system(position())
            .add_system(rotation())
            .add_system(scale())
            .add_system(position_rotation())
            .add_system(position_scale())
            .add_system(rotation_scale())
            .add_system(position_rotation_scale())
            .add_system(inherit_transforms())
        //.add_thread_local(player_transform_shower())
    }
}

// #[allow(dead_code)]
// fn player_transform_shower() -> impl Runnable {
//     SystemBuilder::new("player_transform_shower")
//         .read_component::<Transform>()
//         .with_query(<&Transform>::query())
//         .read_resource::<crate::components::Player>()
//         .read_resource::<crate::components::PlayerCamera>()
//         .write_resource::<GuiContext>()
//         .build(move |_cmd, world, resources, queries| {
//             let (player, player_camera, _) = resources;
//
//             fn display_table(ui: &Ui, table: [[f32; 4]; 4]) {
//                 ui.columns(4, im_str!("a table"), true);
//                 for i in 0..4 {
//                     ui.separator();
//                     for col in table.iter() {
//                         ui.text(format!("{:.2}", col[i]));
//                         ui.next_column();
//                     }
//                 }
//                 ui.columns(1, im_str!("this is dumb"), false);
//                 ui.separator();
//             }
//
//             use imgui::{im_str, Condition};
//             let names = [
//                 im_str!("player"),
//                 im_str!("player model"),
//                 im_str!("camera"),
//             ];
//
//             for i in 0..3 {
//                 let entity = [player.player, player.model, player_camera.entity][i];
//                 GuiContext::with_ui(|ui| {
//                     imgui::Window::new(names[i])
//                         .position([400.0 + i as f32 * 200.0, 50.0], Condition::FirstUseEver)
//                         .size([200.0, 280.0], Condition::FirstUseEver)
//                         .build(ui, || {
//                             if let Ok(trans) = queries.get(world, entity) {
//                                 ui.text("Absolute transform:");
//                                 display_table(ui, trans.absolute.into());
//                                 ui.text("Relative transform:");
//                                 display_table(ui, trans.relative.into());
//                             }
//                         });
//                 });
//             }
//         })
// }

fn populate_transforms() -> impl Runnable {
    SystemBuilder::new("populate_transforms")
        .with_query(<Entity>::query().filter(
            !component::<Transform>()
                & (component::<Position>() | component::<Rotation>() | component::<Scale>()),
        ))
        .build(move |cmd, world, _, query| {
            query.for_each_mut(world, |ent: &Entity| {
                cmd.forge(*ent).transform_identity();
            });
        })
}

fn depopulate_transforms() -> impl Runnable {
    SystemBuilder::new("depopulate_transforms")
        .with_query(<Entity>::query().filter(
            component::<Transform>()
                & !component::<Position>()
                & !component::<Rotation>()
                & !component::<Scale>(),
        ))
        .build(move |cmd, world, _, query| {
            query.for_each_mut(world, |ent: &Entity| {
                cmd.forge(*ent).remove_component::<Transform>();
            });
        })
}

fn adopt_children() -> impl Runnable {
    SystemBuilder::new("adopt_children")
        .with_query(<(Entity, &Parent)>::query().filter(maybe_changed::<Parent>()))
        .read_component::<Parent>()
        .write_component::<Children>()
        .build(move |cmd, world, _, query| {
            query.for_each(world, |(ent, par): (&Entity, &Parent)| {
                cmd.forge(par.0).adopt_child(*ent);
            });
        })
}

// when a transform informing component changes, reset the transform such that it can be recalculated
fn reset_transforms() -> impl Runnable {
    SystemBuilder::new("populate_transforms")
        .write_component::<Transform>()
        .with_query(<&mut Transform>::query().filter(
            maybe_changed::<Position>() | maybe_changed::<Rotation>() | maybe_changed::<Scale>(),
        ))
        .build(move |_, world, _, query| {
            query.for_each_mut(world, |transform: &mut Transform| {
                *transform = Transform::identity();
            });
        })
}

#[allow(dead_code)]
fn calculate_relative_transforms() -> impl ParallelRunnable {
    SystemBuilder::new("transforms_position")
        .write_component::<Transform>()
        .read_component::<Position>()
        .read_component::<Rotation>()
        .read_component::<Scale>()
        .with_query(
            <(&mut Transform, &Position)>::query()
                .filter(!component::<Rotation>() & !component::<Scale>()),
        )
        .with_query(
            <(&mut Transform, &Rotation)>::query()
                .filter(!component::<Position>() & !component::<Scale>()),
        )
        .with_query(
            <(&mut Transform, &Scale)>::query()
                .filter(!component::<Position>() & !component::<Rotation>()),
        )
        .with_query(<(&mut Transform, &Position, &Rotation)>::query().filter(!component::<Scale>()))
        .with_query(<(&mut Transform, &Position, &Scale)>::query().filter(!component::<Rotation>()))
        .with_query(<(&mut Transform, &Rotation, &Scale)>::query().filter(!component::<Position>()))
        .with_query(<(&mut Transform, &Position, &Rotation, &Scale)>::query())
        .build(move |_, world, _, query| {
            let (q0, q1, q2, q3, q4, q5, q6) = query;

            q0.for_each_mut(world, |(transform, val)| {
                transform.relative = transform.relative * Matrix4::from(val);
            });
            q1.for_each_mut(world, |(transform, val)| {
                transform.relative = transform.relative * Matrix4::from(val);
            });
            q2.for_each_mut(world, |(transform, val)| {
                transform.relative = transform.relative * Matrix4::from(val);
            });
            q3.for_each_mut(world, |(transform, val1, val2)| {
                transform.relative = transform.relative * Matrix4::from(val1) * Matrix4::from(val2);
            });
            q4.for_each_mut(world, |(transform, val1, val2)| {
                transform.relative = transform.relative * Matrix4::from(val1) * Matrix4::from(val2);
            });
            q5.for_each_mut(world, |(transform, val1, val2)| {
                transform.relative = transform.relative * Matrix4::from(val1) * Matrix4::from(val2);
            });
            q6.for_each_mut(world, |(transform, val1, val2, val3)| {
                transform.relative = transform.relative
                    * Matrix4::from(val1)
                    * Matrix4::from(val2)
                    * Matrix4::from(val3);
            });
        })
}

macro_rules! transform_system_one {
    ($name:ident, $q:ty, ($a:ty, $b:ty)) => {
        fn $name() -> impl ParallelRunnable {
            SystemBuilder::new(String::from("transforms_") + stringify!($name))
                .read_component::<Transform>() // this isn't technically true but enforces the correct access patterns
                .with_query(
                    <(&mut Transform, $q)>::query().filter(
                        maybe_changed::<Transform>() & !component::<$a>() & !component::<$b>(),
                    ),
                )
                .build(move |_, world, _, query| {
                    query.for_each_mut(world, |(transform, val)| {
                        transform.relative = transform.relative * Matrix4::from(val);
                    });
                })
        }
    };
}
transform_system_one!(position, &Position, (Rotation, Scale));
transform_system_one!(rotation, &Rotation, (Position, Scale));
transform_system_one!(scale, &Scale, (Position, Rotation));

macro_rules! transform_system_two {
    ($name:ident, ($a:ty, $b:ty), $f:ty) => {
        fn $name() -> impl ParallelRunnable {
            SystemBuilder::new(String::from("transforms_") + stringify!($name))
                .read_component::<Transform>() // this isn't technically true but enforces the correct access patterns
                .with_query(
                    <(&mut Transform, $a, $b)>::query()
                        .filter(maybe_changed::<Transform>() & !component::<$f>()),
                )
                .build(move |_, world, _, query| {
                    query.for_each_mut(world, |(transform, val1, val2)| {
                        transform.relative =
                            transform.relative * Matrix4::from(val1) * Matrix4::from(val2);
                    });
                })
        }
    };
}
transform_system_two!(position_rotation, (&Position, &Rotation), Scale);
transform_system_two!(position_scale, (&Position, &Scale), Rotation);
transform_system_two!(rotation_scale, (&Rotation, &Scale), Position);

fn position_rotation_scale() -> impl ParallelRunnable {
    SystemBuilder::new("transforms_rotation_scale")
        .read_component::<Transform>() // this isn't technically true but enforces the correct access patterns
        .with_query(
            <(&mut Transform, &Position, &Rotation, &Scale)>::query()
                .filter(maybe_changed::<Transform>()),
        )
        .build(move |_, world, _, query| {
            query.for_each_mut(world, |(transform, val1, val2, val3)| {
                transform.relative = transform.relative
                    * Matrix4::from(val1)
                    * Matrix4::from(val2)
                    * Matrix4::from(val3);
            });
        })
}

fn inherit_transforms() -> impl ParallelRunnable {
    SystemBuilder::new("inherit_transforms")
        .read_component::<Children>()
        .write_component::<Transform>()
        .with_query(<&mut Transform>::query().filter(!component::<Parent>()))
        .with_query(
            <(Entity, &Transform)>::query()
                .filter(!component::<Parent>() & component::<Children>()),
        )
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
                        if let Ok::<&mut Transform, EntityAccessError>(child_transform) =
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
