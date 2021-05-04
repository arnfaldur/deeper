use application::UnitStage;
use legion::systems::{Builder, ParallelRunnable};
use legion::{Resources, SystemBuilder, World};

use crate::{CommandManager, InputState};

pub struct InputUnit;

impl application::Unit for InputUnit {
    fn load_resources(&self, _: &mut World, resources: &mut Resources) {
        resources.insert(InputState::new());
        resources.insert(CommandManager::default_bindings());
    }
    fn add_systems(&self, stage: UnitStage, builder: &mut Builder) {
        match stage {
            UnitStage::StartFrame => {
                builder.add_system(update_command_manager_system());
            }
            UnitStage::EndFrame => {
                builder.add_system(input_state_new_frame_system());
            }
            _ => (),
        }
    }
}

fn update_command_manager_system() -> impl ParallelRunnable {
    SystemBuilder::new("update_input_state_system")
        .write_resource::<CommandManager>()
        .read_resource::<InputState>()
        .build(move |_, _, (command_manager, input_state), _| {
            command_manager.update(input_state);
        })
}

fn input_state_new_frame_system() -> impl ParallelRunnable {
    SystemBuilder::new("input_state_new_frame_system")
        .write_resource::<InputState>()
        .build(move |_, _, input_state, _| input_state.new_frame())
}
