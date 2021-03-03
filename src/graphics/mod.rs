use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;
use winit::window::Window;
use zerocopy::AsBytes;

use crate::components::{Model3D, StaticModel};
// How dirty of me
use crate::graphics::data::*;
use crate::loader::AssetManager;

pub const COLOR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;
pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

pub const MAX_NR_OF_POINT_LIGHTS: usize = 10;

pub mod canvas;
pub mod data;
pub mod gui;
pub mod rendering_3d;
pub mod util;

use rendering_3d::*;

pub struct ModelQueue {
    pub local_uniforms: Vec<LocalUniforms>,
    pub model_desc: Vec<Model3D>,
    pub static_models: Vec<StaticModel>,
}

impl ModelQueue {
    fn new() -> Self {
        Self {
            local_uniforms: vec![],
            model_desc: vec![],
            static_models: vec![],
        }
    }

    fn push_model(&mut self, model: Model3D, uniforms: LocalUniforms) {
        self.model_desc.push(model);
        self.local_uniforms.push(uniforms);
    }

    fn clear(&mut self) {
        self.local_uniforms.clear();
        self.model_desc.clear();
        self.static_models.clear();
    }
}

/** The graphics context.
    Currently just a grab-bag of all the state and functionality
    needed to power all graphics. Needs simplification.
*/
pub struct Context {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,

    surface: wgpu::Surface,
    swap_chain: wgpu::SwapChain,
    sc_desc: wgpu::SwapChainDescriptor,

    pub model_render_ctx: ModelRenderContext,
    model_queue: ModelQueue,

    pub window_size: PhysicalSize<u32>,

    pub canvas_render_ctx: canvas::CanvasRenderContext,
    pub canvas_queue: canvas::CanvasQueue,
}

impl Context {
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
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
            })
            .await
            .unwrap()
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
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

        // Maybe we should generalize render contexts? 🤔
        let model_render_ctx = ModelRenderContext::new(&device, window_size);
        let canvas_render_ctx = canvas::CanvasRenderContext::new(&device, window_size);

        let context = Context {
            device,
            queue,
            surface,
            swap_chain,
            sc_desc,
            window_size,
            model_queue: ModelQueue::new(),
            model_render_ctx,
            canvas_render_ctx,
            canvas_queue: canvas::CanvasQueue::new(),
        };

        return context;
    }

    pub fn recompile_model_pipeline(
        &mut self,
        vs_module: wgpu::ShaderModule,
        fs_module: wgpu::ShaderModule,
    ) {
        self.model_render_ctx
            .recompile_pipeline(&self.device, vs_module, fs_module);
    }

    pub fn draw_static_model(&mut self, model: StaticModel) {
        self.model_queue.static_models.push(model);
    }

    pub fn draw_model(
        &mut self,
        model: &Model3D,
        transform: cgmath::Matrix4<f32>,
        // position: Vector3<f32>,
        // rotation: Option<Deg<f32>>,
    ) {
        use cgmath::Matrix4;

        let mut matrix = Matrix4::from_scale(model.scale);
        matrix = transform * matrix;
        //
        // if let Some(rot) = rotation {
        //     matrix = Matrix4::from_angle_z(rot) * matrix;
        // }
        //
        // matrix = Matrix4::from_translation(position) * matrix;

        self.model_queue.push_model(
            model.clone(),
            LocalUniforms {
                model_matrix: matrix.into(),
                material: model.material,
            },
        );
    }

    pub fn set_3d_camera(
        &mut self,
        camera: &crate::components::Camera,
        cam_position: cgmath::Vector3<f32>,
        cam_target: cgmath::Vector3<f32>,
    ) {
        self.model_render_ctx.set_3d_camera(
            &self.queue,
            self.window_size,
            camera,
            cam_position,
            cam_target,
        );
    }

    pub fn render(
        &mut self,
        ass_man: &AssetManager,
        gui_context: &mut gui::GuiContext,
        window: &winit::window::Window,
        debug_timer: &mut crate::debug::DebugTimer, // TODO: Revisit
        draw_debug: bool,
    ) {
        let current_frame = self.swap_chain.get_current_frame().unwrap();

        debug_timer.push("Dynamic Models");

        self.model_render_ctx.render(
            &self.device,
            &self.queue,
            ass_man,
            &self.model_queue,
            &current_frame.output.view,
        );

        self.model_queue.clear();

        debug_timer.pop();
        debug_timer.push("Canvas");

        self.canvas_render_ctx.render(
            &self.device,
            &self.queue,
            &self.canvas_queue,
            &current_frame.output.view,
        );

        self.canvas_queue.clear();

        debug_timer.pop();

        gui_context.debug_render(
            window,
            &self.device,
            &self.queue,
            &current_frame.output.view,
            if draw_debug {
                Some(debug_timer.finish())
            } else {
                None
            },
        );
    }

    // Note(Jökull): A step in the right direction, but a bit heavy-handed
    pub fn model_bind_group_from_uniform_data(
        &self,
        local_uniforms: LocalUniforms,
    ) -> (wgpu::Buffer, wgpu::BindGroup) {
        let _uniforms_size = std::mem::size_of::<LocalUniforms>() as u64;

        let uniform_buf = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: local_uniforms.as_bytes(),
                usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            });

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.model_render_ctx.local_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer {
                    buffer: &uniform_buf,
                    offset: 0,
                    size: None,
                },
            }],
        });

        (uniform_buf, bind_group)
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.window_size = size;

        self.sc_desc = util::sc_desc_from_size(size);
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);

        self.model_render_ctx.resize(&self.device, size);

        self.canvas_render_ctx.set_camera(&self.queue, size);
    }
}
