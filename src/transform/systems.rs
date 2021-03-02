use cgmath::{Matrix4, SquareMatrix};
use legion::systems::{Builder, ParallelRunnable};
use legion::{component, maybe_changed, Entity, IntoQuery, SystemBuilder};

use crate::components::Name;
use crate::systems::flush_command_buffer;
use crate::transform::*;

pub(crate) trait TransformBuilderExtender {
    fn add_transform_systems(&mut self) -> &mut Self;
}

impl TransformBuilderExtender for Builder {
    fn add_transform_systems(&mut self) -> &mut Self {
        self.add_system(populate_transforms())
            .add_system(depopulate_transforms())
            .add_system(flush_command_buffer())
            //.add_system(validate_position_transforms())
            .add_system(validate_rotation_transforms())
            .add_system(validate_scale_transforms())
            .add_system(reset_transforms())
            .add_system(position())
            .add_system(rotation())
            .add_system(rotation3d())
            .add_system(scale())
            .add_system(non_uniform_scale())
            .add_system(player_transform_shower())
    }
}

fn player_transform_shower() -> impl ParallelRunnable {
    SystemBuilder::new("player_transform_shower")
        .with_query(<(Entity, &AbsoluteTransform)>::query())
        .read_resource::<crate::components::Player>()
        .build(move |_cmd, world, resources, queries| {
            let player: &crate::components::Player = resources;
            queries.for_each(world, |(ent, abt)| {
                if *ent == player.entity {
                    eprintln!("abt = {:?}", abt.0);
                }
            })
        })
}

fn populate_transforms() -> impl ParallelRunnable {
    SystemBuilder::new("populate_transforms")
        .with_query(<Entity>::query().filter(
            !component::<AbsoluteTransform>()
                & (component::<Position>()
                    | component::<Rotation>()
                    | component::<Rotation3D>()
                    | component::<Scale>()
                    | component::<NonUniformScale>()),
        ))
        .build(move |cmd, world, _, query| {
            query.for_each_mut(world, |ent: &Entity| {
                cmd.add_component(*ent, AbsoluteTransform(Matrix4::<f32>::identity()));
            });
        })
}

fn depopulate_transforms() -> impl ParallelRunnable {
    SystemBuilder::new("depopulate_transforms")
        .with_query(<Entity>::query().filter(
            component::<AbsoluteTransform>()
                & !component::<Position>()
                & !component::<Rotation>()
                & !component::<Rotation3D>()
                & !component::<Scale>()
                & !component::<NonUniformScale>(),
        ))
        .build(move |cmd, world, _, query| {
            query.for_each_mut(world, |ent: &Entity| {
                cmd.remove_component::<AbsoluteTransform>(*ent);
            });
        })
}

// when a transform informing component changes, reset the transform such that it can be recalculated
fn reset_transforms() -> impl ParallelRunnable {
    SystemBuilder::new("populate_transforms")
        .with_query(<&mut AbsoluteTransform>::query().filter(
            maybe_changed::<Position>()
                | maybe_changed::<Rotation>()
                | maybe_changed::<Rotation3D>()
                | maybe_changed::<Scale>()
                | maybe_changed::<NonUniformScale>(),
        ))
        .build(move |_, world, _, query| {
            query.for_each_mut(world, |abs_tran: &mut AbsoluteTransform| {
                abs_tran.0 = Matrix4::identity();
            });
        })
}

fn validate_rotation_transforms() -> impl ParallelRunnable {
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

fn validate_scale_transforms() -> impl ParallelRunnable {
    SystemBuilder::new("validate_scale_transforms")
        .with_query(<(Option<&Name>, &Scale, &NonUniformScale)>::query())
        .build(move |_, world, _, query| {
            query.for_each(world, |(name, _, _)| {
                eprint!("Note: ");
                if let Some(name) = name {
                    eprint!("The entity {} ", name);
                } else {
                    eprint!("an entity ");
                }
                eprintln!("has both Scale and NonUniformScale components");
            })
        })
}

fn position() -> impl ParallelRunnable {
    SystemBuilder::new("transforms_position")
        .with_query(<(&mut AbsoluteTransform, &Position)>::query())
        .build(move |_, world, _, query| {
            query.for_each_mut(world, |(abs_tran, val)| {
                abs_tran.0 = abs_tran.0 * Matrix4::from(val);
            });
        })
}

fn rotation() -> impl ParallelRunnable {
    SystemBuilder::new("transforms_rotation")
        .with_query(<(&mut AbsoluteTransform, &Rotation)>::query())
        .build(move |_, world, _, query| {
            query.for_each_mut(world, |(abs_tran, val)| {
                abs_tran.0 = abs_tran.0 * Matrix4::from(val);
            });
        })
}

fn rotation3d() -> impl ParallelRunnable {
    SystemBuilder::new("transforms_rotation3d")
        .with_query(<(&mut AbsoluteTransform, &Rotation3D)>::query())
        .build(move |_, world, _, query| {
            query.for_each_mut(world, |(abs_tran, val)| {
                abs_tran.0 = abs_tran.0 * Matrix4::from(val);
            });
        })
}

fn scale() -> impl ParallelRunnable {
    SystemBuilder::new("transforms_scale")
        .with_query(<(&mut AbsoluteTransform, &Scale)>::query())
        .build(move |_, world, _, query| {
            query.for_each_mut(world, |(abs_tran, val)| {
                abs_tran.0 = abs_tran.0 * Matrix4::from(val);
            });
        })
}

fn non_uniform_scale() -> impl ParallelRunnable {
    SystemBuilder::new("transforms_non_uniform_scale")
        .with_query(<(&mut AbsoluteTransform, &NonUniformScale)>::query())
        .build(move |_, world, _, query| {
            query.for_each_mut(world, |(abs_tran, val)| {
                abs_tran.0 = abs_tran.0 * Matrix4::from(val);
            });
        })
}
