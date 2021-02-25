// Welcome to crazy-land

/// Did not want to go this way, but basically feel obligated to do so
/// Based on the way that Amethyst integrates imgui into their engine
/// This solves a problem where we must keep a reference to the current
/// imgui::Ui, which is a mutable reference to the imgui context.
/// The ECS resources will take ownership of the reference, which
/// the borrow checker does not like. The current solution as inspired
/// heavily by amethyst is to just bypass the borrow checker entirely
/// and manage a reference to the memory manually.
pub static mut CURRENT_UI: Option<imgui::Ui<'static>> = None;
pub unsafe fn current_ui<'a>() -> Option<&'a imgui::Ui<'a>> { CURRENT_UI.as_ref() }

/// Contains everything necessary to render GUI elements.
///
/// # notes
/// + Currently focuses on Dear-ImGui using imgui-wgpu-rs.
///     - The way I support this is wildly unsafe
/// + May want to expand this in the future to more GUI libraries.
///     - Or just switch libraries entirely
///
pub struct GuiContext {
    pub imgui_ctx: imgui::Context,
    pub imgui_platform: imgui_winit_support::WinitPlatform,
    pub imgui_renderer: imgui_wgpu::Renderer,
}

impl GuiContext {
    pub fn new(window: &winit::window::Window, context: &crate::graphics::Context) -> Self {
        let mut imgui_ctx = imgui::Context::create();
        let mut imgui_platform = imgui_winit_support::WinitPlatform::init(&mut imgui_ctx);

        imgui_platform.attach_window(
            imgui_ctx.io_mut(),
            window,
            imgui_winit_support::HiDpiMode::Default,
        );

        imgui_ctx.set_ini_filename(None);

        // Font configuration
        let font_size = (13.0 * window.scale_factor()) as f32;
        imgui_ctx.io_mut().font_global_scale = (1.0 / window.scale_factor()) as f32;

        imgui_ctx
            .fonts()
            .add_font(&[imgui::FontSource::DefaultFontData {
                config: Some(imgui::FontConfig {
                    oversample_h: 1,
                    pixel_snap_h: true,
                    size_pixels: font_size,
                    ..Default::default()
                }),
            }]);

        // Renderer setup
        let imgui_renderer = imgui_wgpu::Renderer::new(
            &mut imgui_ctx,
            &context.device,
            &context.queue,
            imgui_wgpu::RendererConfig {
                texture_format: crate::graphics::COLOR_FORMAT,
                ..Default::default()
            },
        );

        return Self {
            imgui_ctx,
            imgui_platform,
            imgui_renderer,
        };
    }

    pub fn render(
        &mut self,
        window: &winit::window::Window,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        view: &wgpu::TextureView,
    ) {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("ImGui Command Encoder"),
        });

        unsafe {
            if let Some(ui) = current_ui() {
                self.imgui_platform.prepare_render(&ui, &window);
            } else {
                panic!("Attempt to render ImGui with no valid Ui reference!");
            }
        }

        let draw_data = unsafe {
            CURRENT_UI = None;
            imgui::sys::igRender();
            &*(imgui::sys::igGetDrawData() as *mut imgui::DrawData)
        };

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });

        self.imgui_renderer
            .render(draw_data, queue, device, &mut render_pass)
            .expect("Rendering failed");

        drop(render_pass);

        queue.submit(std::iter::once(encoder.finish()));
    }

    pub fn wants_input(&self) -> bool {
        let io = self.imgui_ctx.io();
        io.want_capture_mouse || io.want_capture_keyboard || io.want_text_input
    }

    pub fn prep_frame(&mut self, window: &winit::window::Window) {
        self.imgui_platform
            .prepare_frame(self.imgui_ctx.io_mut(), window);
    }

    pub fn new_frame(&mut self) {
        // Don't let mom know
        unsafe {
            CURRENT_UI = Some(std::mem::transmute(self.imgui_ctx.frame()));
        }
    }

    pub fn with_ui(f: impl FnOnce(&imgui::Ui)) {
        unsafe {
            if let Some(ui) = current_ui() {
                (f)(ui);
            }
        }
    }

    pub fn handle_event(
        &mut self,
        window: &mut winit::window::Window,
        event: &winit::event::Event<()>,
    ) {
        self.imgui_platform
            .handle_event(self.imgui_ctx.io_mut(), window, event);
    }
}
