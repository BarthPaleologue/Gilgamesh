use crate::core::app::{App, prepare_frame, render_image};
use crate::core::color_pass::ColorPass;
use crate::core::graph_executor::{FrameTarget, GraphExecutor};
use crate::core::wgpu_context::WGPUContext;
use crate::core::window_surface::WindowSurface;
use crate::input::mouse::Mouse;
use std::sync::Arc;
use web_time::Instant;
use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalSize, Size};
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowAttributes, WindowId};

/// Owns an application and executor, and drives them through the window event loop.
pub struct WindowRuntime<Application: App> {
    app: Application,
    window: Arc<Window>,
    window_surface: WindowSurface,

    executor: GraphExecutor,
    frame: FrameTarget,
    window_title: String,
    image_blit: ColorPass,
    ldr_source: wgpu::BindGroup,
    clock: Instant,
    elapsed_seconds: f64,

    mouse: Mouse,
}

impl<Application: App> WindowRuntime<Application> {
    /// Creates the window and application, then drives rendering and presentation.
    ///
    /// # Panics
    /// Panics if logging or the event loop cannot be initialized, or the event loop fails.
    pub fn run(width: u32, height: u32)
    where
        Application: 'static,
    {
        cfg_if::cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                std::panic::set_hook(Box::new(console_error_panic_hook::hook));
                console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
            } else {
                env_logger::init();
            }
        }

        let event_loop = EventLoop::<Self>::with_user_event()
            .build()
            .expect("Failed to create the window event loop");
        let attributes = Window::default_attributes()
            .with_inner_size(Size::Physical(PhysicalSize { width, height }));
        #[cfg(target_arch = "wasm32")]
        let attributes = {
            use winit::platform::web::WindowAttributesExtWebSys;
            attributes.with_append(true)
        };
        let application = WindowLoop {
            attributes,
            state: ApplicationState::New,
            #[cfg(target_arch = "wasm32")]
            proxy: event_loop.create_proxy(),
        };

        #[cfg(not(target_arch = "wasm32"))]
        {
            let mut application = application;
            event_loop
                .run_app(&mut application)
                .expect("Window event loop failed");
        }

        #[cfg(target_arch = "wasm32")]
        {
            use winit::platform::web::EventLoopExtWebSys;
            event_loop.spawn_app(application);
        }
    }

    async fn new(window: Arc<Window>, display: winit::event_loop::OwnedDisplayHandle) -> Self {
        let instance = wgpu::util::new_instance_with_webgpu_detection(
            wgpu::InstanceDescriptor::new_with_display_handle(Box::new(display)),
        )
        .await;
        let surface = instance
            .create_surface(window.clone())
            .expect("Failed to create a presentation surface for the window");
        let wgpu_context = WGPUContext::new(instance, Some(&surface)).await;
        let window_surface = WindowSurface::new(&wgpu_context, surface, window.inner_size());
        let executor = GraphExecutor::from_context(wgpu_context);
        let frame =
            executor.create_target(window_surface.config.width, window_surface.config.height);
        let ctx = executor.wgpu_context();
        let image_blit = ColorPass::image_blit(&ctx.device, window_surface.config.format);
        let ldr_source = image_blit.bind_source(&ctx.device, frame.output());
        let app = Application::new(ctx);
        let window_title = app.name().to_owned();
        window.set_title(&window_title);

        WindowRuntime {
            app,
            executor,
            frame,
            window_title,
            image_blit,
            ldr_source,
            window,
            window_surface,
            clock: Instant::now(),
            elapsed_seconds: 0.0,
            mouse: Mouse::new(),
        }
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.window_surface
            .resize(&self.executor.wgpu_context().device, new_size);
    }

    fn manage_event(&mut self, event: &WindowEvent) {
        self.mouse.listen_to_event(event);
        self.app.on_window_event(
            event,
            &mut WindowContext {
                executor: &mut self.executor,
                frame: &self.frame,
                elapsed_seconds: self.elapsed_seconds,
            },
        );
        if let WindowEvent::Resized(physical_size) = event {
            self.resize(*physical_size);
        }
    }

    fn render(&mut self, output: wgpu::SurfaceTexture) {
        let size = output.texture.size();
        let previous_output = self.frame.output().clone();
        self.executor
            .resize_target(&mut self.frame, size.width, size.height);
        let elapsed_seconds = self.clock.elapsed().as_secs_f64();
        let delta_seconds = elapsed_seconds - self.elapsed_seconds;
        self.elapsed_seconds = elapsed_seconds;
        let color_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        prepare_frame(
            &mut self.app,
            &self.executor,
            &self.frame,
            elapsed_seconds,
            delta_seconds,
            &self.mouse,
        );
        let title = self.app.name();
        if title != self.window_title {
            self.window.set_title(title);
            self.window_title.clear();
            self.window_title.push_str(title);
        }
        let mut encoder = self.executor.wgpu_context().device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("Window Frame Encoder"),
            },
        );
        self.executor
            .encode(self.app.render_graph(), &mut self.frame, &mut encoder)
            .expect("Render graph execution failed");
        if previous_output != *self.frame.output() {
            self.ldr_source = self
                .image_blit
                .bind_source(&self.executor.wgpu_context().device, self.frame.output());
        }
        self.image_blit
            .encode(&mut encoder, &self.ldr_source, &color_view);
        self.executor
            .wgpu_context()
            .queue
            .submit(std::iter::once(encoder.finish()));
        self.mouse.reset();
        self.window.pre_present_notify();
        self.executor.wgpu_context().queue.present(output);
    }
}

/// Rendering services available to an application's window event callback.
pub struct WindowContext<'a> {
    pub executor: &'a mut GraphExecutor,
    pub frame: &'a FrameTarget,
    pub elapsed_seconds: f64,
}

impl WindowContext<'_> {
    /// Reads the last main frame without invoking application hooks or drawing again.
    pub fn capture_frame(
        &self,
    ) -> impl std::future::Future<Output = std::io::Result<image::RgbaImage>> + use<> {
        self.frame.read_image(self.executor.wgpu_context())
    }

    /// Prepares and renders a separate image, preserving the main frame and simulation time.
    /// The application receives the requested dimensions, zero delta, and neutral mouse input.
    /// Preparation and rendering happen immediately; the returned future only waits for the pixels.
    pub fn render_image<Application: App>(
        &mut self,
        app: &mut Application,
        width: u32,
        height: u32,
    ) -> impl std::future::Future<Output = std::io::Result<image::RgbaImage>> + use<Application>
    {
        render_image(app, self.executor, self.elapsed_seconds, width, height)
    }
}

enum ApplicationState<Application: App> {
    New,
    Initializing,
    Running(Box<WindowRuntime<Application>>),
}

struct WindowLoop<Application: App + 'static> {
    attributes: WindowAttributes,
    state: ApplicationState<Application>,
    #[cfg(target_arch = "wasm32")]
    proxy: winit::event_loop::EventLoopProxy<WindowRuntime<Application>>,
}

impl<Application: App + 'static> ApplicationHandler<WindowRuntime<Application>>
    for WindowLoop<Application>
{
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if !matches!(self.state, ApplicationState::New) {
            return;
        }
        self.state = ApplicationState::Initializing;

        let window = Arc::new(
            event_loop
                .create_window(self.attributes.clone())
                .expect("Failed to create the application window"),
        );
        let display = event_loop.owned_display_handle();

        #[cfg(not(target_arch = "wasm32"))]
        self.user_event(
            event_loop,
            pollster::block_on(WindowRuntime::new(window, display)),
        );

        #[cfg(target_arch = "wasm32")]
        {
            let proxy = self.proxy.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let runtime = WindowRuntime::new(window, display).await;
                let _ = proxy.send_event(runtime);
            });
        }
    }

    fn user_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        mut runtime: WindowRuntime<Application>,
    ) {
        let size = runtime.window.inner_size();
        runtime.resize(size);
        runtime.window.request_redraw();
        self.state = ApplicationState::Running(Box::new(runtime));
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if matches!(event, WindowEvent::CloseRequested) {
            event_loop.exit();
            return;
        }

        let ApplicationState::Running(runtime) = &mut self.state else {
            return;
        };
        if window_id != runtime.window.id() {
            return;
        }

        runtime.manage_event(&event);

        match event {
            WindowEvent::KeyboardInput { event, .. }
                if event.state == ElementState::Pressed
                    && event.physical_key == PhysicalKey::Code(KeyCode::Escape) =>
            {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                use wgpu::CurrentSurfaceTexture;
                match runtime.window_surface.surface.get_current_texture() {
                    CurrentSurfaceTexture::Success(output) => runtime.render(output),
                    CurrentSurfaceTexture::Suboptimal(output) => {
                        runtime.render(output);
                        runtime.resize(runtime.window.inner_size());
                    }
                    CurrentSurfaceTexture::Outdated => {
                        runtime.resize(runtime.window.inner_size());
                    }
                    CurrentSurfaceTexture::Lost => {
                        runtime.window_surface.recreate_surface(
                            runtime.executor.wgpu_context(),
                            runtime.window.clone(),
                        );
                    }
                    CurrentSurfaceTexture::Timeout | CurrentSurfaceTexture::Occluded => {}
                    CurrentSurfaceTexture::Validation => {
                        log::error!("Surface texture validation failed");
                    }
                }
                runtime.window.request_redraw();
            }
            _ => {}
        }
    }
}
