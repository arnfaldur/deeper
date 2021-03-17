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
    //pub textures_path: PathBuf,
    //pub shader_path: PathBuf,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Extensions {
    pub models: Vec<String>,
    //pub textures: Vec<String>,
    //pub shaders: Vec<String>,
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

impl Default for DisplaySettings {
    fn default() -> Self {
        Self {
            screen_width: 1024,
            screen_height: 768,
            fps: 60,
        }
    }
}

#[derive(Clone, Debug)]
pub struct StorageInfo<T> {
    pub id: T,
    pub loaded_at_time: SystemTime,
}

impl<T> StorageInfo<T> {
    pub fn now(id: T) -> Option<Self> {
        Some(Self {
            id,
            loaded_at_time: SystemTime::now(),
        })
    }
}

#[derive(Clone, Debug)]
pub enum AssetStorageInfo {
    Model(Option<StorageInfo<graphics::ModelID>>),
    //Texture(Option<StorageInfo<graphics::TextureID>>),
    //Shader(Option<StorageInfo<graphics::ShaderID>>),
    Unrecognized,
}

#[derive(Clone)]
pub struct Asset {
    pub file_name: String,
    pub path: PathBuf,
    pub storage_info: AssetStorageInfo,
}
