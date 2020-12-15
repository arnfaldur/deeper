use serde::{Deserialize, Serialize};

use std::collections::HashMap;

use zerocopy::AsBytes;

use std::fs;
use std::fs::File;
use std::io::Read;
use itertools::Itertools;

use std::time::{SystemTime};

use wavefront_obj::obj;
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

    // TODO: implement mark_changed_files(...)

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
                                self.load_model(&path, &ext, context);
                            }
                        }
                    }
                }
            }
        }
    }

    // Assumes is a valid model
    fn load_model(&mut self, path: &Path, ext: &String, context: &graphics::Context) {
        if let Some(
            Asset{
                loaded_at_time: time_loaded,
                asset_kind: AssetKind::Model(idx), ..
            }) = self.assets.get(path) {
            let modified = fs::metadata(path).unwrap().modified().unwrap();
            if modified.gt(&time_loaded) {
                self.models[*idx] = AssetManager::get_graphics_model(path, ext, context);
                println!("[loader] Hotloaded: {:?}", path.file_name().unwrap());
                self.assets.get_mut(path).unwrap().loaded_at_time = SystemTime::now();
            }
        } else {
            self.models.push(AssetManager::get_graphics_model(path, ext, context));
            println!("[loader] Loaded: {:?}", path.file_name().unwrap());
            self.register_asset(path, AssetKind::Model(self.models.len() - 1));
        }
    }

    // Note(Jökull): The graphics layer should possibly deal with this :shrug:
    fn get_graphics_model(path: &Path, ext: &String, context: &graphics::Context) -> graphics::Model {
        // TODO: Generalize this
        match ext.as_str() {
            "obj"          => load_model_from_obj(context, path),
            "glb" | "gltf" => load_model_from_gltf(context, path),
            _ => {
                // Should not happen
                println!("[loader] (error): Extension {} not recognized.", ext);
                graphics::Model { meshes: vec![] }
            }
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


// Model file format handling
// Note(Jökull): We really only need the device, so...
// TODO: Grab only the device from context, not the whole context

fn load_model_from_gltf(context: &graphics::Context, path: &Path) -> graphics::Model {
    let vertex_lists = vertex_lists_from_gltf(path).unwrap();
    return load_model_from_vertex_lists(context, &vertex_lists);
}

fn load_model_from_obj(context: &graphics::Context, path: &Path) -> graphics::Model {
    let vertex_lists = vertex_lists_from_obj(path).unwrap();
    return load_model_from_vertex_lists(context, &vertex_lists);
}

fn load_model_from_vertex_lists(context: &graphics::Context, vertex_lists: &Vec<Vec<graphics::Vertex>>) -> graphics::Model {
    let mut meshes = vec!();

    for vertices in vertex_lists {

        let vertex_buf = context.device.create_buffer_with_data(
            vertices.as_bytes(),
            wgpu::BufferUsage::VERTEX,
        );

        meshes.push(
            graphics::Mesh {
                num_vertices: vertices.len(),
                vertex_buffer: vertex_buf,
                offset: [0.0, 0.0, 0.0],
            }
        );
    }

    graphics::Model { meshes }
}

// TODO: Handle transforms
pub fn vertex_lists_from_gltf(path: &Path) -> Result<Vec<Vec<graphics::Vertex>>, String> {
    let (document, buffers, _images) = gltf::import(path)
        .expect(format!("[graphics/gltf] : File {} could not be opened", path.display()).as_ref());

    // TODO: Add checks for multiple models/scenes, etc.

    let mut vertex_lists = vec!();

    for mesh in document.meshes() {
        let mut vertex_list = vec!();
        for primitive in mesh.primitives() {
            // TODO: Is there a more readable way to do this
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

            // TODO: This feels ... wrong
            let positions = reader.read_positions().unwrap().collect_vec();
            let normals = reader.read_normals().unwrap().collect_vec();
            // TODO: What is set?
            let tex_coords = reader.read_tex_coords(0).unwrap().into_f32().collect_vec();

            let indices = reader.read_indices().unwrap().into_u32();

            for idx in indices {
                let pos = positions.get(idx as usize).unwrap().clone();
                let normal = normals.get(idx as usize).unwrap().clone();
                let tex_coord = tex_coords.get(idx as usize).unwrap().clone();

                vertex_list.push(graphics::Vertex { pos, normal, tex_coord })
            }
        }
        vertex_lists.push(vertex_list);
    }

    return Ok(vertex_lists);
}

pub fn vertex_lists_from_obj(path: &Path) -> Result<Vec<Vec<graphics::Vertex>>, String> {

    let mut f;

    if let Ok(file) = File::open(path) {
        f = file;
    } else {
        return Err(format!("[graphics] : File {} could not be opened.", path.display()));
    };

    let mut buf = String::new();
    f.read_to_string(&mut buf);

    let obj_set = obj::parse(buf)
        .expect("Failed to parse obj file");

    let mut vertex_lists = vec!();

    for obj in &obj_set.objects {

        let mut vertices = vec!();

        for geometry in &obj.geometry {
            let mut indices = vec!();

            geometry.shapes.iter().for_each(|shape| {
                if let obj::Primitive::Triangle(v1, v2, v3) = shape.primitive {
                    indices.push(v1);
                    indices.push(v2);
                    indices.push(v3);
                }
            });

            for idx in &indices {
                let pos = obj.vertices[idx.0];

                let normal = match idx.2 {
                    Some(i) => obj.normals[i],
                    _ => obj::Normal{ x: 0.0, y: 0.0, z: 0.0}
                };

                let tc = match idx.1 {
                    Some(i) => obj.tex_vertices[i],
                    _ => obj::TVertex{ u: 0.0, v: 0.0, w: 0.0}
                };

                let v = graphics::Vertex {
                    pos:        [pos.x as f32, pos.y as f32, pos.z as f32],
                    normal:     [normal.x as f32, normal.y as f32, normal.z as f32],
                    tex_coord:  [tc.u as f32, tc.v as f32]
                };
                vertices.push(v);
            }
        }
        vertex_lists.push(vertices);
    }
    Ok(vertex_lists)
}
