//! Optional egui integration. Forward window events, declare widgets, then draw [`egui_overlay`].
//! UI preparation is independent of the GPU; graph execution preserves the prepared frame for captures.

pub use egui;
mod overlay;
mod renderer;
mod textures;
pub use overlay::egui_overlay;

use egui::epaint::{ClippedPrimitive, TextureId};
use gilgamesh::core::WindowContext;
use renderer::GpuRenderer;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;
use textures::TextureSnapshot;
use winit::{event::WindowEvent, window::Window};

struct WindowBridge {
    window: Arc<Window>,
    state: egui_winit::State,
}

/// One root viewport. GPU resources are prepared by [`egui_overlay`] for its actual target.
/// A prepared UI frame can be rendered repeatedly without running widget callbacks again.
#[must_use]
pub struct Egui {
    context: egui::Context,
    window: Option<WindowBridge>,
    renderer: RefCell<Option<GpuRenderer>>,
    paint_jobs: Vec<ClippedPrimitive>,
    pixels_per_point: f32,
    textures: HashMap<TextureId, TextureSnapshot>,
    free_textures: Vec<TextureId>,
    generation: u64,
}

impl Default for Egui {
    fn default() -> Self {
        Self {
            context: egui::Context::default(),
            window: None,
            renderer: RefCell::new(None),
            paint_jobs: Vec::new(),
            pixels_per_point: 1.0,
            textures: HashMap::new(),
            free_textures: Vec::new(),
            generation: 0,
        }
    }
}

impl Egui {
    #[must_use]
    pub fn context(&self) -> &egui::Context {
        &self.context
    }

    /// Forwards native input and suppresses scene input/shortcuts consumed by egui.
    /// Initializes the window bridge on the first event.
    ///
    /// # Panics
    /// Panics if the context has no native window. Use [`Self::run_with_input`] for headless UI.
    #[expect(
        clippy::cast_possible_truncation,
        reason = "egui uses f32 pixels per point"
    )]
    pub fn on_window_event(&mut self, event: &WindowEvent, context: &mut WindowContext<'_>) {
        let window = context
            .window
            .expect("egui window events need a native window");
        let bridge = self.window.get_or_insert_with(|| WindowBridge {
            window: window.clone(),
            state: egui_winit::State::new(
                self.context.clone(),
                egui::ViewportId::ROOT,
                window.as_ref(),
                Some(window.scale_factor() as f32),
                window.theme(),
                Some(
                    context
                        .executor
                        .wgpu_context()
                        .device
                        .limits()
                        .max_texture_dimension_2d as usize,
                ),
            ),
        });
        let response = bridge.state.on_window_event(&bridge.window, event);
        if response.consumed {
            context.consume_event();
        }
        if response.repaint {
            bridge.window.request_redraw();
        }
    }

    /// Declares the current frame's widgets and handles cursor, clipboard and IME output.
    /// A callback may run several times if egui requests another layout pass.
    ///
    /// # Panics
    /// Panics before the first [`Self::on_window_event`] call.
    pub fn run(&mut self, ui: impl FnMut(&egui::Context)) {
        let bridge = self
            .window
            .as_mut()
            .expect("forward window events before running egui");
        let output = self
            .context
            .run_pass(bridge.state.take_egui_input(&bridge.window), ui);
        bridge
            .state
            .handle_platform_output(&bridge.window, output.platform_output);
        self.prepare(
            output.shapes,
            output.pixels_per_point,
            output.textures_delta,
        );
    }

    /// Runs a UI frame containing one window with egui's default settings.
    /// Use [`Self::run`] for multiple windows, panels or custom window settings.
    ///
    /// # Panics
    /// Panics before the first [`Self::on_window_event`] call.
    pub fn run_window(
        &mut self,
        title: impl Into<egui::WidgetText>,
        mut ui: impl FnMut(&mut egui::Ui),
    ) {
        let title = title.into();
        self.run(|context| {
            egui::Window::new(title.clone()).show(context, |contents| ui(contents));
        });
    }

    /// Prepares widgets from explicit input, without a native window or GPU.
    /// Returns platform output for applications that provide their own input backend.
    pub fn run_with_input(
        &mut self,
        input: egui::RawInput,
        ui: impl FnMut(&egui::Context),
    ) -> egui::PlatformOutput {
        let output = self.context.run_pass(input, ui);
        self.prepare(
            output.shapes,
            output.pixels_per_point,
            output.textures_delta,
        );
        output.platform_output
    }

    fn prepare(
        &mut self,
        shapes: Vec<egui::epaint::ClippedShape>,
        pixels_per_point: f32,
        mut delta: egui::TexturesDelta,
    ) {
        self.generation += 1;
        // Retire textures only when their prepared frame is replaced, allowing capture replays.
        for id in self.free_textures.drain(..) {
            self.textures.remove(&id);
        }
        for (id, updates) in delta.set.drain() {
            for update in updates {
                textures::update(&mut self.textures, id, update, self.generation);
            }
        }
        self.free_textures = delta.free.drain().collect();
        self.paint_jobs = self.context.tessellate(shapes, pixels_per_point);
        self.pixels_per_point = pixels_per_point;
    }
}
