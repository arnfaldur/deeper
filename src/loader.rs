use serde::{Deserialize, Serialize};
use notify::{RecommendedWatcher, Watcher, RecursiveMode, DebouncedEvent};
use std::collections::HashMap;
use std::fs;
use std::time::{SystemTime, Duration};
use std::sync::mpsc::{channel, Sender, Receiver};
use crate::graphics;

#[derive(Serialize, Deserialize)]
struct PathSettings {
    display_settings_path: PathBuf,
    extensions_settings_path: PathBuf,
    assets_path: PathBuf,
    models_path: PathBuf,
    entities_path: PathBuf,
    shaders_path: PathBuf,
}

impl PathSettings {
    fn new() -> Self {
        Self {
            display_settings_path: Path::new("settings/display.settings").to_path_buf(),
            extensions_settings_path: Path::new("settings/extensions.settings").to_path_buf(),
            assets_path: Path::new("assets/").to_path_buf(),
            models_path: Path::new("assets/Models/").to_path_buf(),
            entities_path: Path::new("entities/").to_path_buf(),
            shaders_path: Path::new("shaders/").to_path_buf(),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Extensions {
    models : Vec<String>,
}

impl Extensions {
    fn new() -> Self {
        Self {
            models: vec![],
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DisplaySettings {
    pub screen_width: i32,
    pub screen_height: i32,
    pub fps: u32,
}

impl DisplaySettings {
    pub fn new() -> Self {
        Self {
            screen_width: 1024,
            screen_height: 768,
            fps: 60,
        }
    }
}

enum AssetKind {
    Settings,
    Model(usize),
}

struct Asset {
    name: String,
    changed: bool,
    loaded_at_time: SystemTime,
    asset_kind: AssetKind,
}

pub struct AssetManager {
    assets: HashMap<PathBuf, Asset>,
    paths: PathSettings,
    extensions: Extensions,

    pub models: Vec<graphics::Model>,
}

use std::path::Path;
use std::path::PathBuf;
use std::borrow::Borrow;
use crate::loader::AssetKind::Model;

impl AssetManager {
    pub fn new() -> Self {
        let ps_path = Path::new("settings/paths.settings");
        let mut paths = PathSettings::new();

        if fs::metadata(ps_path).is_ok() {
            let data = fs::read_to_string(ps_path).unwrap();
            paths = ron::de::from_str(data.as_str()).unwrap();
        } else {
            eprintln!("No path settings found at path: \"{}\"", ps_path.display());
        }

        let ext_path = paths.extensions_settings_path.clone();

        let mut ass_man = Self {
            assets: Default::default(),
            paths,
            extensions: Extensions::new(),
            models: vec!(),
        };

        ass_man.register_extensions(ext_path.as_ref());
        ass_man.register_asset(ps_path, AssetKind::Settings);

        return ass_man;
    }

    pub fn mark_changed_files(&mut self, changes: Vec<PathBuf>) {
        unimplemented!();
    }

    pub fn get_model_index(&self, name: &str ) -> Option<usize> {
        if let Some(asset) = self.assets.values().find(|model|
            model.name == name.to_string()) {
            match asset.asset_kind {
                Model(idx) => Some(idx),
                _ => None,
            }
        } else {
            None
        }
    }

    pub fn load_models(&mut self, context: &graphics::Context) {
        self.load_models_recursive(self.paths.models_path.clone().as_ref(), context);
    }

    fn load_models_recursive(&mut self, path: &Path, context: &graphics::Context) {
        let model_extensions = self.extensions.models.clone();
        for dir_entry in fs::read_dir(path).unwrap() {
            if let Ok(entry) = dir_entry {
                let path = entry.path().clone();
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_dir() {
                        self.load_models_recursive(&path, context);
                    } else if file_type.is_file() {
                        if let Some(ext) = entry.path().extension() {
                            let ext = ext.to_str().unwrap().to_string();
                            if model_extensions.contains(&ext) {
                                self.load_model(&path, context);
                            }
                        }
                    }
                }
            }
        }
    }

    fn load_model(&mut self, path: &Path, context: &graphics::Context) {
        // Assumes is a valid model
        if let Some(
            Asset{
                loaded_at_time: time_loaded,
                asset_kind: AssetKind::Model(idx), ..
            }) = self.assets.get(path) {
            let modified = fs::metadata(path).unwrap().modified().unwrap();
            if modified.gt(&time_loaded) {
                self.models[*idx] = context.load_model_from_obj(path);
                println!("[loader] Hotloaded: {:?}", path.file_name().unwrap());
                self.assets.get_mut(path).unwrap().loaded_at_time = SystemTime::now();
            }
        } else {
            let model = context.load_model_from_obj(path);
            self.models.push(model);
            println!("[loader] Loaded: {:?}", path.file_name().unwrap());
            self.register_asset(path, AssetKind::Model(self.models.len() - 1));
        }
    }

    fn update_time_loaded(&mut self, resource: &Path) {
        if self.assets.contains_key(resource) {}
    }

    fn register_asset(&mut self, path: &Path, asset_kind: AssetKind) {
        let name = path.file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        self.assets.insert(path.to_path_buf(), Asset {
            name,
            changed: false,
            loaded_at_time: SystemTime::now(),
            asset_kind
        });
    }

    fn register_extensions(&mut self, path: &Path) {
        if fs::metadata(path).is_ok() {
            let data = fs::read_to_string(path).unwrap();
            self.extensions = ron::de::from_str(data.as_str()).unwrap();
            self.register_asset(path, AssetKind::Settings);
        } else {
            eprintln!("Error reading extensions: \"{}\"", path.display());
        }
    }

    pub fn load_display_settings(&mut self) -> DisplaySettings {
        let ds_path = self.paths.display_settings_path.clone();
        if fs::metadata(ds_path.as_path()).is_ok() {
            let data = fs::read_to_string(ds_path.as_path()).unwrap();
            //self.loaded_at_time.insert(ds_path, SystemTime::now());
            self.register_asset(ds_path.as_ref(), AssetKind::Settings);
            return ron::de::from_str(data.as_str()).unwrap()
        } else {
            eprintln!("No display settings found at path: \"{}\"", ds_path.display());
        }
        return DisplaySettings::new();
    }


}

