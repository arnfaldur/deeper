use std::path::Path;
use std::time::SystemTime;

use legion::*;

use crate::input::{Command, CommandManager};
use crate::{assets, graphics};

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
