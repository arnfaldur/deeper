use assman::GraphicsAssetManager;
use graphics;
use input::{Command, CommandManager};
use legion::*;

#[system]
pub fn hot_loading(
    //#[state] shaders_loaded_at: &mut SystemTime,
    #[state] hotload_shaders_turned_on: &mut bool,
    #[resource] asset_store: &mut assman::AssetStore,
    #[resource] graphics_context: &mut graphics::GraphicsContext,
    #[resource] graphics_resources: &mut graphics::GraphicsResources,
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
        GraphicsAssetManager::new(asset_store, graphics_resources, graphics_context).load_models();
    }

    //if *hotload_shaders_turned_on {
    //    //asset_store.load_shaders(shaders_loaded_at, context)
    //}
}
