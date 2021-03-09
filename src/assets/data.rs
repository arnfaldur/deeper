#![allow(unused)]

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct PathSettings {
    pub display_settings_path: PathBuf,
    pub extensions_settings_path: PathBuf,
    pub assets_path: PathBuf,
    pub models_path: PathBuf,
    pub entities_path: PathBuf,
    pub shaders_path: PathBuf,
}

impl PathSettings {
    pub fn new() -> Self {
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

#[derive(Serialize, Deserialize, Default)]
pub struct Extensions {
    pub models: Vec<String>,
    pub images: Vec<String>,
}

impl Extensions {
    pub fn new() -> Self { Self::default() }
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

pub enum AssetKind {
    Settings,
    Model(usize),
}

pub struct Asset {
    pub file_name: String,
    pub path: PathBuf,
    pub loaded_at_time: SystemTime,
    pub asset_kind: AssetKind,
}
