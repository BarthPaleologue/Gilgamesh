use crate::core::app::{App, FrameContext};
use crate::core::renderer::Renderer;
use crate::core::wgpu_context::WGPUContext;
use crate::input::mouse::Mouse;
use std::sync::Arc;
use web_time::Instant;
use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalSize, Size};
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowAttributes, WindowId};

pub struct Engine<Application: App> {
    pub app: Application,
    pub window: Arc<Window>,
    pub wgpu_context: WGPUContext,

    renderer: Renderer,
    clock: Instant,
    elapsed_seconds: f64,
    delta_seconds: f64,

    mouse: Mouse,
}

impl<Application: App> Engine<Application> {
    /// Initializes WebGPU, creates the application once, then runs the event loop.
    pub fn run(name: &str, width: u32, height: u32)
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

        let event_loop = EventLoop::<Self>::with_user_event().build().unwrap();
        let attributes = Window::default_attributes()
            .with_inner_size(Size::Physical(PhysicalSize { width, height }))
            .with_title(name);
        #[cfg(target_arch = "wasm32")]
        let attributes = {
            use winit::platform::web::WindowAttributesExtWebSys;
            attributes.with_append(true)
        };
        let application = EngineApplication {
            attributes,
            state: ApplicationState::New,
            #[cfg(target_arch = "wasm32")]
            proxy: event_loop.create_proxy(),
        };

        #[cfg(not(target_arch = "wasm32"))]
        {
            let mut application = application;
            event_loop.run_app(&mut application).unwrap();
        }

        #[cfg(target_arch = "wasm32")]
        {
            use winit::platform::web::EventLoopExtWebSys;
            event_loop.spawn_app(application);
        }
    }

    async fn new(window: Arc<Window>, display: winit::event_loop::OwnedDisplayHandle) -> Self {
        let wgpu_context = WGPUContext::new(window.clone(), display).await;

        let renderer = Renderer::new(&wgpu_context);
        let app = Application::new(&wgpu_context);

        Engine {
            app,
            renderer,
            window,
            wgpu_context,
            clock: Instant::now(),
            elapsed_seconds: 0.0,
            delta_seconds: 0.0,
            mouse: Mouse::new(),
        }
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.wgpu_context.config.width = new_size.width;
            self.wgpu_context.config.height = new_size.height;
            self.wgpu_context
                .surface
                .configure(&self.wgpu_context.device, &self.wgpu_context.config);
        }
    }

    fn manage_event(&mut self, event: &WindowEvent) {
        self.mouse.listen_to_event(event);
        self.app.on_window_event(event);
        if let WindowEvent::Resized(physical_size) = event {
            self.resize(*physical_size);
        }
    }

    fn render(&mut self, output: wgpu::SurfaceTexture) {
        let frame = FrameContext {
            elapsed_seconds: self.elapsed_seconds,
            delta_seconds: self.delta_seconds,
            surface_width: self.wgpu_context.config.width,
            surface_height: self.wgpu_context.config.height,
            mouse: &self.mouse,
            wgpu_context: &self.wgpu_context,
        };
        self.app.update_before_render(&frame);
        self.renderer
            .render(self.app.get_render_context(), &self.wgpu_context, &output);
        self.mouse.reset();
        self.window.pre_present_notify();
        self.wgpu_context.queue.present(output);
    }
}

enum ApplicationState<Application: App> {
    New,
    Initializing,
    Running(Box<Engine<Application>>),
}

struct EngineApplication<Application: App + 'static> {
    attributes: WindowAttributes,
    state: ApplicationState<Application>,
    #[cfg(target_arch = "wasm32")]
    proxy: winit::event_loop::EventLoopProxy<Engine<Application>>,
}

impl<Application: App + 'static> ApplicationHandler<Engine<Application>>
    for EngineApplication<Application>
{
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if !matches!(self.state, ApplicationState::New) {
            return;
        }
        self.state = ApplicationState::Initializing;

        let window = Arc::new(event_loop.create_window(self.attributes.clone()).unwrap());
        let display = event_loop.owned_display_handle();

        #[cfg(not(target_arch = "wasm32"))]
        self.user_event(event_loop, pollster::block_on(Engine::new(window, display)));

        #[cfg(target_arch = "wasm32")]
        {
            let proxy = self.proxy.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let engine = Engine::new(window, display).await;
                let _ = proxy.send_event(engine);
            });
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, mut engine: Engine<Application>) {
        let size = engine.window.inner_size();
        engine.resize(size);
        engine.window.request_redraw();
        self.state = ApplicationState::Running(Box::new(engine));
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

        let ApplicationState::Running(engine) = &mut self.state else {
            return;
        };
        if window_id != engine.window.id() {
            return;
        }

        engine.manage_event(&event);

        match event {
            WindowEvent::KeyboardInput { event, .. }
                if event.state == ElementState::Pressed
                    && event.physical_key == PhysicalKey::Code(KeyCode::Escape) =>
            {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                let new_elapsed_seconds = engine.clock.elapsed().as_secs_f64();
                engine.delta_seconds = new_elapsed_seconds - engine.elapsed_seconds;
                engine.elapsed_seconds = new_elapsed_seconds;

                use wgpu::CurrentSurfaceTexture;
                match engine.wgpu_context.surface.get_current_texture() {
                    CurrentSurfaceTexture::Success(output) => engine.render(output),
                    CurrentSurfaceTexture::Suboptimal(output) => {
                        engine.render(output);
                        engine.resize(engine.window.inner_size());
                    }
                    CurrentSurfaceTexture::Outdated => {
                        engine.resize(engine.window.inner_size());
                    }
                    CurrentSurfaceTexture::Lost => {
                        engine.wgpu_context.recreate_surface(engine.window.clone());
                    }
                    CurrentSurfaceTexture::Timeout | CurrentSurfaceTexture::Occluded => {}
                    CurrentSurfaceTexture::Validation => {
                        log::error!("Surface texture validation failed");
                    }
                }
                engine.window.request_redraw();
            }
            _ => {}
        }
    }
}
