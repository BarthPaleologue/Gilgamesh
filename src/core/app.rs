use crate::core::graph_executor::{FrameTarget, GraphExecutor};
use crate::core::render_graph::RenderGraph;
use crate::core::wgpu_context::WGPUContext;
use crate::core::window_runtime::WindowContext;
use crate::input::mouse::Mouse;
use winit::event::WindowEvent;

pub struct FrameContext<'a> {
    pub elapsed_seconds: f64,
    pub delta_seconds: f64,
    /// Render target width in physical pixels.
    pub render_width: u32,
    /// Render target height in physical pixels.
    pub render_height: u32,
    pub mouse: &'a Mouse,
    pub wgpu_context: &'a WGPUContext,
}

pub trait App: Sized {
    /// Creates the application after WebGPU has been initialized.
    fn new(ctx: &WGPUContext) -> Self;

    /// Current display name, used by the window runtime after each frame preparation.
    fn name(&self) -> &str;

    /// Prepares the application and camera for this render's dimensions and time.
    fn update_before_render(&mut self, _frame: &FrameContext<'_>) {}

    /// Describes this render, borrowing any scenes and parameters it needs.
    /// Export one color image at the render dimensions for presentation and capture.
    fn render_graph(&self) -> RenderGraph<'_>;

    fn on_window_event(&mut self, _event: &WindowEvent, _context: &mut WindowContext<'_>) {}
}

pub(crate) fn prepare_frame(
    app: &mut impl App,
    executor: &GraphExecutor,
    target: &FrameTarget,
    elapsed_seconds: f64,
    delta_seconds: f64,
    mouse: &Mouse,
) {
    let texture = target.output().texture();
    app.update_before_render(&FrameContext {
        elapsed_seconds,
        delta_seconds,
        render_width: texture.width(),
        render_height: texture.height(),
        mouse,
        wgpu_context: executor.wgpu_context(),
    });
}

pub(crate) fn render_image<Application: App>(
    app: &mut Application,
    executor: &mut GraphExecutor,
    elapsed_seconds: f64,
    width: u32,
    height: u32,
) -> impl std::future::Future<Output = std::io::Result<image::RgbaImage>> + use<Application> {
    let mut target = executor.create_target(width, height);
    prepare_frame(app, executor, &target, elapsed_seconds, 0.0, &Mouse::new());
    executor
        .execute(app.render_graph(), &mut target)
        .expect("Render graph execution failed");
    target.read_image(executor.wgpu_context())
}
