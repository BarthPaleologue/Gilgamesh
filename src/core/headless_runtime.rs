use crate::core::app::{App, prepare_frame, render_image};
use crate::core::frame_renderer::{FrameRenderer, FrameTarget};
use crate::core::wgpu_context::WGPUContext;
use crate::input::mouse::Mouse;
use std::future::Future;
use std::time::Duration;

/// Owns an application and renderer for rendering without a window or event loop.
pub struct HeadlessRuntime<Application: App> {
    app: Application,
    renderer: FrameRenderer,
    frame: FrameTarget,
    elapsed_seconds: f64,
}

impl<Application: App> HeadlessRuntime<Application> {
    /// Initializes the runtime and executes the callback with ownership of it.
    /// Blocks until completion on native platforms; schedules a browser task on WASM.
    pub fn run<Task, TaskFuture>(width: u32, height: u32, task: Task)
    where
        Application: 'static,
        Task: FnOnce(Self) -> TaskFuture + 'static,
        TaskFuture: Future<Output = ()> + 'static,
    {
        #[cfg(target_arch = "wasm32")]
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        let run = async move {
            task(Self::new(width, height).await).await;
        };
        #[cfg(not(target_arch = "wasm32"))]
        pollster::block_on(run);
        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(run);
    }

    pub async fn new(width: u32, height: u32) -> Self {
        let renderer = FrameRenderer::new().await;
        let frame = renderer.create_target(width, height);
        let app = Application::new(renderer.wgpu_context());
        Self {
            app,
            renderer,
            frame,
            elapsed_seconds: 0.0,
        }
    }

    pub fn app(&self) -> &Application {
        &self.app
    }

    pub fn app_mut(&mut self) -> &mut Application {
        &mut self.app
    }

    pub fn wgpu_context(&self) -> &WGPUContext {
        self.renderer.wgpu_context()
    }

    /// Advances by the caller's timestep and renders the main frame with neutral input.
    pub fn render(&mut self, delta: Duration) {
        let delta_seconds = delta.as_secs_f64();
        self.elapsed_seconds += delta_seconds;
        prepare_frame(
            &mut self.app,
            &self.renderer,
            &self.frame,
            self.elapsed_seconds,
            delta_seconds,
            &Mouse::new(),
        );
        self.renderer.render_to(
            &self.frame,
            self.app.get_frame_bindings_data(),
            self.app.get_meshes(),
        );
    }

    /// Replaces the main target when its dimensions change; rendering remains explicit.
    pub fn resize(&mut self, width: u32, height: u32) {
        self.renderer.resize_target(&mut self.frame, width, height);
    }

    pub fn output(&self) -> &wgpu::TextureView {
        self.frame.output()
    }

    /// Starts reading the last main frame without calling application hooks again.
    pub fn capture_frame(
        &self,
    ) -> impl std::future::Future<Output = std::io::Result<image::RgbaImage>> + use<Application>
    {
        self.frame.read_image(self.renderer.wgpu_context())
    }

    /// Renders a separate image at the current time, with zero delta and neutral input.
    pub fn render_image(
        &mut self,
        width: u32,
        height: u32,
    ) -> impl std::future::Future<Output = std::io::Result<image::RgbaImage>> + use<Application>
    {
        render_image(
            &mut self.app,
            &mut self.renderer,
            self.elapsed_seconds,
            width,
            height,
        )
    }
}
