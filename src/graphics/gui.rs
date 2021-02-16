// Welcome to crazy-land

/// Did not want to go this way, but basically feel obligated to do so
/// Based on the way that Amethyst integrates imgui into their engine
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
