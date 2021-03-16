use std::time::SystemTime;

use graphics;
use graphics::assets;
use legion::*;

use input::{Command, CommandManager};

#[system]
pub fn hot_loading(
    #[state] shaders_loaded_at: &mut SystemTime,
    #[state] hotload_shaders_turned_on: &mut bool,
    #[resource] ass_man: &mut assets::AssetManager,
    #[resource] context: &mut graphics::Context,
    #[resource] input: &CommandManager,
) {
    if input.get(Command::DevToggleHotLoading) {
        println!(
            "Hotloading shaders turned {}",
            if *hotload_shaders_turned_on {
                "OFF"
            } else {
                "ON"
            }
        );
        *hotload_shaders_turned_on = !*hotload_shaders_turned_on;
    }

    if input.get(Command::DevHotLoadModels) {
        println!("Hotloading models...");
        ass_man.load_models(context);
    }

    if *hotload_shaders_turned_on {
        ass_man.load_shaders(shaders_loaded_at, context)
    }
}
