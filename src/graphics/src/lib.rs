#![feature(slice_group_by)]
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;
use winit::window::Window;

pub const COLOR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8Unorm;
pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

pub const MAX_NR_OF_POINT_LIGHTS: usize = 10;

pub mod canvas;
pub mod components;
pub mod data;
pub mod debug;
pub mod gui;
pub mod models;
pub mod systems;
pub mod util;

use std::collections::HashMap;
use std::sync::Arc;

use slotmap::SlotMap;

use crate::data::Vertex;

pub type ModelID = slotmap::DefaultKey;
pub type TextureID = slotmap::DefaultKey;
pub type ShaderID = String;

pub struct GraphicsResources {
    pub models: SlotMap<ModelID, data::Model>,
    pub textures: SlotMap<TextureID, data::Texture>,
    pub shaders: HashMap<ShaderID, Arc<wgpu::ShaderModule>>,
}

impl GraphicsResources {
    pub fn new() -> Self {
        Self {
            models: SlotMap::new(),
            textures: SlotMap::new(),
            shaders: HashMap::new(),
        }
    }
}

pub struct RenderContext<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub current_frame: Arc<wgpu::SwapChainFrame>,
    pub window_size: PhysicalSize<u32>,
}

pub struct GraphicsContext {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,

    surface: wgpu::Surface,
    swap_chain: wgpu::SwapChain,
    sc_desc: wgpu::SwapChainDescriptor,
    pub window_size: PhysicalSize<u32>,
}

impl GraphicsContext {
    pub async fn new(window: &Window) -> Self {
        let window_size = window.inner_size();

        // This creates a wgpu instance. We use this to create an Adapter and a Surface
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        // A surface is a platform-specific target that you can render images onto
        let surface = unsafe { instance.create_surface(window) };
        // The device represents the GPU essentially
        // and the queue represents a command queue
        // present on the GPU
        let (device, queue) = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: None,
            })
            .await
            .unwrap()
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::default(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .unwrap();

        // The swap_chain represents the images that will be presented to our surface.
        // You ask the swap_chain for the current frame that is being rendered to
        // and when you drop it, the swap chain will present the frame to the surface.
        let sc_desc = util::sc_desc_from_size(window_size);
        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        Self {
            device,
            queue,
            surface,
            swap_chain,
            sc_desc,
            window_size,
        }
    }

    pub fn begin_render(&self) -> RenderContext {
        RenderContext {
            device: &self.device,
            queue: &self.queue,
            current_frame: Arc::new(self.swap_chain.get_current_frame().unwrap()),
            window_size: self.window_size,
        }
    }

    pub fn model_from_vertex_list(&self, vertex_lists: Vec<Vec<Vertex>>) -> data::Model {
        let mut meshes = vec![];

        for vertices in vertex_lists.iter() {
            let vertex_buf = self
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(vertices.as_slice()),
                    usage: wgpu::BufferUsage::VERTEX,
                });

            meshes.push(data::Mesh {
                num_vertices: vertices.len(),
                vertex_buffer: vertex_buf,
                offset: [0.0, 0.0, 0.0],
            });
        }

        data::Model {
            meshes,
            vertex_lists,
        }
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.window_size = size;

        self.sc_desc = util::sc_desc_from_size(size);
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
    }
}
