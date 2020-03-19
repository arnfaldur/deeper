use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;
use std::fs;

#[derive(Serialize, Deserialize)]
struct PathSettings {
    display_settings_path : String,
    assets_path : String,
    entities_path : String,
    shaders_path : String,
}

impl PathSettings {
    fn new() -> Self {
        Self {
            display_settings_path: "settings/display.settings".to_string(),
            assets_path: "assets/".to_string(),
            entities_path: "entities/".to_string(),
            shaders_path: "shaders/".to_string()
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DisplaySettings {
    pub screen_width  : i32,
    pub screen_height : i32,

    pub fps       : u32,
    pub tile_size : i32,
    pub max_tiles : f64, // TODO: what is this?
}

impl DisplaySettings {
    pub fn new() -> Self {
        Self {
            screen_width: 1024,
            screen_height: 768,
            fps: 60,
            tile_size: 10,
            max_tiles: 64.0
        }
    }
}

pub struct AssetManager {
    loaded_at_time: HashMap<String, SystemTime>,
    paths : PathSettings,
}

impl AssetManager {
    pub fn new() -> Self {
        Self { loaded_at_time: Default::default(), paths: PathSettings::new() }.initialize()
    }

    fn initialize(mut self) -> Self {
        let ps_path = "settings/paths.settings";
        if fs::metadata(ps_path).is_ok() {
            let data = fs::read_to_string(ps_path).unwrap();
            self.loaded_at_time.insert(ps_path.parse().unwrap(), SystemTime::now());
            self.paths = serde_json::from_str(data.as_str()).unwrap();
        } else {
            eprintln!("No path settings found at path: \"{}\"", ps_path);
        }
        return self;
    }

    pub fn load_display_settings(mut self) -> DisplaySettings {
        let ds_path = self.paths.display_settings_path;
        if fs::metadata(ds_path.as_str()).is_ok() {
            let data = fs::read_to_string(ds_path.as_str()).unwrap();
            self.loaded_at_time.insert(ds_path,SystemTime::now());
            return serde_json::from_str(data.as_str()).unwrap();
        } else {
            eprintln!("No display settings found at path: \"{}\"", ds_path);
        }
        return DisplaySettings::new();
    }
}
