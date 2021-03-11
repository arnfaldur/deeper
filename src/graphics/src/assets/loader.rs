use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use super::data::*;
use crate::assets::reader;
use crate::{data, Context};

pub struct AssetManager {
    assets: HashMap<PathBuf, Asset>,
    paths: PathSettings,
    extensions: Extensions,

    pub models: Vec<crate::data::Model>,
}

impl AssetManager {
    pub fn new() -> Self {
        let paths = reader::read_ron::<PathSettings>("settings/paths.settings".as_ref()).unwrap();

        let extensions = reader::read_ron::<Extensions>(&paths.extensions_settings_path).unwrap();

        Self {
            assets: Default::default(),
            paths,
            extensions,
            models: vec![],
        }
    }

    pub fn get_model_index(&self, name: &str) -> Option<usize> {
        if let Some(asset) = self
            .assets
            .values()
            .find(|model| model.file_name == name.to_string())
        {
            match asset.asset_kind {
                AssetKind::Model(idx) => Some(idx),
                _ => None,
            }
        } else {
            None
        }
    }

    pub fn load_models(&mut self, context: &mut Context) {
        self.load_models_recursive(self.paths.models_path.clone().as_ref(), context);
    }

    fn load_models_recursive(&mut self, path: &Path, context: &mut Context) {
        for dir_entry in fs::read_dir(path).unwrap() {
            if let Ok(entry) = dir_entry {
                let path = entry.path().clone();
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_dir() {
                        self.load_models_recursive(&path, context);
                    } else if file_type.is_file() {
                        if let Some(extension) = entry.path().extension() {
                            let extension = extension.to_str().unwrap().to_string();
                            if self.extensions.models.contains(&extension) {
                                self.load_model(&path, &extension, context);
                            }
                        }
                    }
                }
            }
        }
    }

    // Assumes is a valid model
    fn load_model(&mut self, path: &Path, ext: &String, context: &mut Context) {
        if let Some(Asset {
            loaded_at_time: time_loaded,
            asset_kind: AssetKind::Model(idx),
            ..
        }) = self.assets.get_mut(path)
        {
            let modified = fs::metadata(path).unwrap().modified().unwrap();
            if modified.gt(time_loaded) {
                *time_loaded = SystemTime::now();
                self.models[*idx] = AssetManager::get_graphics_model(path, ext, context);
                println!("[loader] Hot-loaded: {:?}", path.file_name().unwrap());
            }
        } else {
            self.register_asset(path, AssetKind::Model(self.models.len()));
            self.models
                .push(AssetManager::get_graphics_model(path, ext, context));
            println!("[loader] Loaded: {:?}", path.file_name().unwrap());
        }
    }

    pub fn allocate_graphics_model_from_vertex_lists(
        &mut self,
        context: &mut Context,
        vertex_lists: data::VertexLists,
    ) -> usize {
        let idx = self.models.len();
        self.models
            .push(context.model_from_vertex_list(vertex_lists));
        idx
    }

    fn get_graphics_model(path: &Path, ext: &String, context: &mut Context) -> data::Model {
        // TODO: Generalize this
        context.model_from_vertex_list(match ext.as_str() {
            "obj" => super::reader::vertex_lists_from_obj(path).unwrap(),
            "glb" | "gltf" => super::reader::vertex_lists_from_gltf(path).unwrap(),
            _ => {
                // Should not happen
                eprintln!("[loader] (error): Extension {} not recognized.", ext);
                vec![]
            }
        })
    }

    fn register_asset(&mut self, path: &Path, asset_kind: AssetKind) {
        let file_name = path.file_name().unwrap().to_str().unwrap().to_string();
        self.assets.insert(
            path.to_path_buf(),
            Asset {
                file_name,
                path: path.to_path_buf(),
                loaded_at_time: SystemTime::now(),
                asset_kind,
            },
        );
    }

    pub fn load_display_settings(&mut self) -> DisplaySettings {
        let ds_path = self.paths.display_settings_path.clone();
        if fs::metadata(ds_path.as_path()).is_ok() {
            let data = fs::read_to_string(ds_path.as_path()).unwrap();
            //self.loaded_at_time.insert(ds_path, SystemTime::now());
            self.register_asset(ds_path.as_ref(), AssetKind::Settings);
            return ron::de::from_str(data.as_str()).unwrap();
        } else {
            eprintln!(
                "No display settings found at path: \"{}\"",
                ds_path.display()
            );
        }
        return DisplaySettings::new();
    }

    pub fn load_shaders(&mut self, shaders_loaded_at: &mut SystemTime, context: &mut Context) {
        let frag_path = Path::new("shaders/forward.frag");
        let vert_path = Path::new("shaders/forward.vert");

        let frag_modified = std::fs::metadata(frag_path).unwrap().modified().unwrap();
        let vert_modified = std::fs::metadata(vert_path).unwrap().modified().unwrap();

        let mut compiler = shaderc::Compiler::new().unwrap();

        if frag_modified.gt(shaders_loaded_at) || vert_modified.gt(shaders_loaded_at) {
            let vs_mod = if let Ok(data) = std::fs::read_to_string(vert_path) {
                if let Ok(vs_spirv) = compiler.compile_into_spirv(
                    data.as_str(),
                    shaderc::ShaderKind::Vertex,
                    "forward.vert",
                    "main",
                    None,
                ) {
                    Some(
                        context
                            .device
                            .create_shader_module(&wgpu::ShaderModuleDescriptor {
                                label: Some("Vertex Shader"),
                                source: wgpu::util::make_spirv(&vs_spirv.as_binary_u8()),
                                flags: wgpu::ShaderFlags::VALIDATION,
                            }),
                    )
                } else {
                    eprintln!("Failed to recompile vertex shader");
                    None
                }
            } else {
                eprintln!("Failed to read vertex shader");
                None
            };

            let fs_mod = if let Ok(data) = std::fs::read_to_string(frag_path) {
                if let Ok(fs_spirv) = compiler.compile_into_spirv(
                    data.as_str(),
                    shaderc::ShaderKind::Fragment,
                    "forward.frag",
                    "main",
                    None,
                ) {
                    Some(
                        context
                            .device
                            .create_shader_module(&wgpu::ShaderModuleDescriptor {
                                label: Some("Fragment Shader"),
                                source: wgpu::util::make_spirv(&fs_spirv.as_binary_u8()),
                                flags: wgpu::ShaderFlags::VALIDATION,
                            }),
                    )
                } else {
                    eprintln!("Failed to recompile fragment shader");
                    None
                }
            } else {
                eprintln!("Failed to read fragment shader");
                None
            };

            if let (Some(vsm), Some(fsm)) = (vs_mod, fs_mod) {
                println!("Recompiling shaders...");
                context.recompile_model_pipeline(&vsm, &fsm);
                *shaders_loaded_at = SystemTime::now();
            }
        }
    }
}
