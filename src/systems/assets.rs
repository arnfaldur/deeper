use specs::prelude::*;

use crate::{graphics, loader};
use crate::input::{InputState, Key};

use std::time::SystemTime;
use std::path::Path;
use glsl_to_spirv::ShaderType;

pub struct HotLoaderSystem {
    // pub watcher: RecommendedWatcher,
    // pub rx: Receiver<DebouncedEvent>,
    pub shaders_loaded_at: SystemTime,
    pub hotload_shaders_turned_on: bool,
}

impl HotLoaderSystem {
    pub fn new() -> Self {
        Self { shaders_loaded_at: SystemTime::now(), hotload_shaders_turned_on: false }
    }
}

impl<'a> System<'a> for HotLoaderSystem {
    type SystemData = (
        WriteExpect<'a, loader::AssetManager>,
        WriteExpect<'a, graphics::Context>,
        ReadExpect<'a, InputState>,
    );

    fn run(&mut self, (mut ass_man, mut context, input): Self::SystemData) {
        if input.is_key_pressed(Key::H) {
            println!("Hotloading shaders turned {}", if self.hotload_shaders_turned_on { "OFF" } else { "ON" });
            self.hotload_shaders_turned_on = !self.hotload_shaders_turned_on;
        }

        if input.is_key_pressed(Key::L) {
            println!("Hotloading models...");
            ass_man.load_models(&context);
        }

        if self.hotload_shaders_turned_on {
            let frag_path = Path::new("shaders/forward.frag");
            let vert_path = Path::new("shaders/forward.vert");

            let frag_modified = std::fs::metadata(frag_path).unwrap().modified().unwrap();
            let vert_modified = std::fs::metadata(vert_path).unwrap().modified().unwrap();

            if frag_modified.gt(&self.shaders_loaded_at) || vert_modified.gt(&self.shaders_loaded_at) {
                let vs_mod = if let Ok(data) = std::fs::read_to_string(vert_path) {
                    if let Ok(vs) = glsl_to_spirv::compile(data.as_str(), ShaderType::Vertex) {
                        if let Ok(sprv) = &wgpu::read_spirv(vs) {
                            Some(context.device.create_shader_module(sprv))
                        } else {
                            eprintln!("Failed to create shader module");
                            None
                        }
                    } else {
                        eprintln!("Failed to recompile vertex shader");
                        None
                    }
                } else {
                    eprintln!("Failed to read vertex shader");
                    None
                };

                let fs_mod = if let Ok(data) = std::fs::read_to_string(frag_path) {
                    if let Ok(fs) = glsl_to_spirv::compile(data.as_str(), ShaderType::Fragment) {
                        if let Ok(sprv) = &wgpu::read_spirv(fs) {
                            Some(context.device.create_shader_module(sprv))
                        } else {
                            eprintln!("Failed to create shader module");
                            None
                        }
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
                    context.recompile_pipeline(vsm, fsm);
                    self.shaders_loaded_at = SystemTime::now();
                }
            }
        }
    }
}
