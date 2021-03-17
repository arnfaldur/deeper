use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::data::*;
use super::reader;

//pub const DEFAULT_SETTINGS_PATH: &'static str = "settings/";
//pub const PATHS_SETTINGS_NAME: &'static str = "paths.settings";

pub struct AssetStore {
    assets: HashMap<PathBuf, Asset>,
    paths: PathSettings,
    extensions: Extensions,
}

impl AssetStore {
    pub fn init() -> Self {
        let paths = reader::read_ron::<PathSettings>("settings/paths.settings".as_ref()).unwrap();

        let extensions = reader::read_ron::<Extensions>(&paths.extensions_settings_path).unwrap();

        Self {
            assets: Default::default(),
            paths,
            extensions,
        }
    }

    pub fn register_assets(&mut self, path: Option<&Path>) {
        let path = path.unwrap_or(&self.paths.assets_path);

        fs::read_dir(path)
            .unwrap()
            .filter_map(|x| x.ok())
            .map(|e| {
                let file_type = e.file_type().unwrap();

                if file_type.is_dir() {
                    self.register_assets(Some(&e.path()));
                } else if file_type.is_file() {
                    if !self.assets.contains_key(&e.path()) {
                        self.register_asset(
                            &e.path(),
                            self.new_asset_storage_info_from_ext(
                                &e.path()
                                    .extension()
                                    .unwrap()
                                    .to_str()
                                    .unwrap_or("")
                                    .to_string(),
                            ),
                        );
                    }
                }
            })
            .count(); // Consume
    }

    // Temporary evil
    pub fn get_model_index(&self, name: &str) -> Option<graphics::ModelID> {
        if let Some(x) = self
            .assets
            .values()
            .find(|p| p.file_name == name.to_string())
            .map(|f| f.storage_info.clone())
        {
            if let AssetStorageInfo::Model(Some(y)) = x {
                Some(y.id)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn new_asset_storage_info_from_ext(&self, ext: &String) -> AssetStorageInfo {
        if self.extensions.models.contains(ext) {
            AssetStorageInfo::Model(None)
        //} else if self.extensions.textures.contains(ext) {
        //    AssetStorageInfo::Texture(None)
        //} else if self.extensions.shaders.contains(ext) {
        //    AssetStorageInfo::Shader(None)
        } else {
            AssetStorageInfo::Unrecognized
        }
    }

    fn register_asset(&mut self, path: &Path, asset_storage_info: AssetStorageInfo) {
        let file_name = path.file_name().unwrap().to_str().unwrap().to_string();
        match asset_storage_info {
            AssetStorageInfo::Unrecognized => (),
            _ => {
                self.assets.insert(
                    path.to_path_buf(),
                    Asset {
                        file_name,
                        path: path.to_owned(),
                        storage_info: asset_storage_info,
                    },
                );
            }
        };
    }

    pub fn load_display_settings(&mut self) -> DisplaySettings {
        reader::read_ron::<DisplaySettings>(&self.paths.display_settings_path).unwrap_or({
            println!(
                "Failed to load DisplaySettings at path: {:?}",
                self.paths.display_settings_path
            );
            DisplaySettings::default()
        })
    }
}

pub struct GraphicsAssetManager<'a, 'b, 'c> {
    asset_store: &'a mut AssetStore,
    graphics_resources: &'b mut graphics::GraphicsResources,
    graphics_context: &'c mut graphics::GraphicsContext,
}

impl<'a, 'b, 'c> GraphicsAssetManager<'a, 'b, 'c> {
    pub fn new(
        asset_store: &'a mut AssetStore,
        graphics_resources: &'b mut graphics::GraphicsResources,
        graphics_context: &'c mut graphics::GraphicsContext,
    ) -> Self {
        Self {
            asset_store,
            graphics_resources,
            graphics_context,
        }
    }

    pub fn load_asset(&mut self, path: &Path) -> Option<Asset> {
        match self.asset_store.new_asset_storage_info_from_ext(
            &path.extension().unwrap().to_str().unwrap().to_string(),
        ) {
            AssetStorageInfo::Model(..) => self.load_model(path),
            //AssetStorageInfo::Texture(..) => self.load_texture(path),
            //AssetStorageInfo::Shader(..) => self.load_shader(path),
            AssetStorageInfo::Unrecognized => None,
        }
    }

    pub fn load_assets_recursive(&mut self, path: &Path) {
        fs::read_dir(path)
            .unwrap()
            .filter_map(|x| x.ok())
            .map(|e| {
                let file_type = e.file_type().unwrap();

                if file_type.is_dir() {
                    self.load_assets_recursive(&e.path());
                } else if file_type.is_file() {
                    if self.asset_store.assets.contains_key(&e.path()) {
                        self.load_asset(&e.path());
                    }
                }
            })
            .count();
    }

    pub fn get_asset_info(&self, file_name: &str) -> Option<Asset> {
        self.asset_store
            .assets
            .values()
            .find(|asset| asset.file_name == file_name.to_string())
            .map(|e| e.clone())
    }

    //fn load_shader(&mut self, path: &Path) -> Option<Asset> {
    //    let file_name = path.file_name().unwrap().to_str().unwrap();
    //    let ext = path.extension().unwrap().to_str().unwrap();

    //    let mut shader_compiler = shaderc::Compiler::new().unwrap();

    //    if let Ok(spirv) = shader_compiler.compile_into_spirv(
    //        &fs::read_to_string(path).unwrap(),
    //        match ext {
    //            "frag" => shaderc::ShaderKind::Fragment,
    //            "vert" => shaderc::ShaderKind::Vertex,
    //            &_ => {
    //                eprintln!("Invalid shader extension: {}", &ext);
    //                shaderc::ShaderKind::InferFromSource
    //            }
    //        },
    //        file_name,
    //        "main",
    //        None,
    //    ) {
    //        let shader_module =
    //            self.graphics_context
    //                .device
    //                .create_shader_module(&wgpu::ShaderModuleDescriptor {
    //                    label: None,
    //                    source: wgpu::util::make_spirv(spirv.as_binary_u8()),
    //                    flags: Default::default(),
    //                });
    //        if let Some(Asset {
    //                        asset_storage_info: AssetStorageInfo::Shader(Some(storage_info)),
    //                        ..
    //                    }) = self.asset_store.assets.get_mut(path)
    //        {
    //            *storage_info.loaded_at_time = SystemTime::now();
    //            *self
    //                .graphics_resources
    //                .shaders
    //                .get_mut(&storage_info.id)
    //                .unwrap() = Arc::new(shader_module);
    //        } else {
    //            let id = file_name.to_string();
    //            self.graphics_resources
    //                .shaders
    //                .insert(id.clone(), Arc::new(shader_module));
    //            self.asset_store
    //                .register_asset(path, AssetStorageInfo::Shader(StorageInfo::now(id)));
    //        }
    //    }

    //    *self.asset_store.assets.get(path)
    //}

    //fn load_texture(&mut self, path: &Path) -> Option<Asset> {
    //    if let Some(Asset {
    //                    asset_storage_info: AssetStorageInfo::Image(Some(storage_info)),
    //                    ..
    //                }) = self.asset_store.assets.get_mut(path)
    //    {
    //        if let Some(image) = reader::read_image(path) {
    //            *storage_info.time_loaded = SystemTime::now();
    //            *self
    //                .graphics_resources
    //                .textures
    //                .get_mut(storage_info.id)
    //                .unwrap() = graphics::data::Texture::new(image, self.graphics_context);
    //        } else {
    //            println!("Failed to load: {}", path.display());
    //        }
    //    } else {
    //        if let Some(image) = reader::read_image(path) {
    //            let id = self
    //                .graphics_resources
    //                .textures
    //                .insert(graphics::data::Texture::new(image, self.graphics_context));
    //            self.asset_store
    //                .register_asset(path, AssetStorageInfo::Image(StorageInfo::now(id)));
    //        } else {
    //            println!("Failed to load: {}", path.display());
    //        }
    //    }

    //    *self.asset_store.assets.get(path)
    //}

    fn load_model(&mut self, path: &Path) -> Option<Asset> {
        let ext = path.extension().unwrap().to_str().unwrap();

        let asset_entry = self.asset_store.assets.get_mut(path).cloned();

        let mut exists = false;

        if let Some(mut asset) = asset_entry {
            if let AssetStorageInfo::Model(Some(storage_info)) = asset.storage_info.clone() {
                exists = true;
                let model = self.get_graphics_model(path, ext);
                self.graphics_resources
                    .models
                    .insert(storage_info.id, model);
                asset.storage_info = AssetStorageInfo::Model(StorageInfo::now(storage_info.id));
                self.asset_store.assets.insert(path.to_owned(), asset);
            }
        }
        if !exists {
            let id = self.graphics_resources.models.len();

            let model = self.get_graphics_model(path, ext);

            self.graphics_resources.models.push(model);

            self.asset_store
                .register_asset(path, AssetStorageInfo::Model(StorageInfo::now(id)));
        }

        self.asset_store.assets.get(path).map(|f| f.clone())
    }

    pub fn load_models(&mut self) {
        self.load_assets_recursive(&self.asset_store.paths.models_path.clone());
    }

    //pub fn load_images(&mut self, context: &mut graphics::GraphicsContext) {
    //    self.load_assets_recursive(&self.asset_store.paths.textures_path.clone(), context);
    //}

    pub fn allocate_graphics_model_from_vertex_lists(
        &mut self,
        vertex_lists: graphics::data::VertexLists,
    ) -> graphics::ModelID {
        let ret = self.graphics_resources.models.len();
        self.graphics_resources
            .models
            .push(self.graphics_context.model_from_vertex_list(vertex_lists));

        ret
    }

    fn get_graphics_model(&mut self, path: &Path, ext: &str) -> graphics::data::Model {
        // TODO: Generalize this
        self.graphics_context.model_from_vertex_list(match ext {
            "obj" => super::reader::vertex_lists_from_obj(path).unwrap(),
            "glb" | "gltf" => super::reader::vertex_lists_from_gltf(path).unwrap(),
            _ => {
                // Should not happen
                eprintln!("[loader] (error): Extension {} not recognized.", ext);
                vec![]
            }
        })
    }
}
