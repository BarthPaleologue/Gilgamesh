use crate::core::app::{App, FrameContext};
use crate::core::wgpu_context::WGPUContext;
use crate::input::mouse::Mouse;
use std::sync::Arc;
use std::time::SystemTime;
use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalSize, Size};
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowAttributes, WindowId};

pub struct Engine<TApp: App> {
    pub app: TApp,
    pub window: Arc<Window>,
    pub wgpu_context: WGPUContext,

    clock: SystemTime,
    elapsed_seconds: f32,
    delta_seconds: f32,

    mouse: Mouse,
}

impl<TApp: App> Engine<TApp> {
    /// Initializes WebGPU, calls the application factory once, then runs the event loop.
    pub fn run(
        name: &str,
        width: u32,
        height: u32,
        create_app: impl FnOnce(&WGPUContext) -> TApp + 'static,
    ) where
        TApp: 'static,
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
        let application = EngineApplication {
            attributes,
            state: ApplicationState::New(create_app),
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

    async fn new(
        window: Arc<Window>,
        display: winit::event_loop::OwnedDisplayHandle,
        create_app: impl FnOnce(&WGPUContext) -> TApp,
    ) -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            // Winit prevents sizing with CSS, so we have to set
            // the size manually when on web.
            use winit::dpi::PhysicalSize;
            let _ = window.request_inner_size(PhysicalSize::new(450, 400));

            use winit::platform::web::WindowExtWebSys;
            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| {
                    let dst = doc.get_element_by_id("wasm-example")?;
                    let canvas = web_sys::Element::from(window.canvas()?);
                    dst.append_child(&canvas).ok()?;
                    Some(())
                })
                .expect("Couldn't append canvas to document body.");
        }

        let wgpu_context = WGPUContext::new(window.clone(), display).await;

        let app = create_app(&wgpu_context);

        Engine {
            app,
            window,
            wgpu_context,
            clock: SystemTime::now(),
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
            self.app.get_render_context().camera.aspect_ratio =
                new_size.width as f32 / new_size.height as f32;
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
            mouse: &self.mouse,
            wgpu_context: &self.wgpu_context,
        };
        self.app.update_before_render(&frame);
        self.app
            .get_render_context()
            .render(&self.wgpu_context, &output);
        self.mouse.reset();
        self.window.pre_present_notify();
        self.wgpu_context.queue.present(output);
    }
}

enum ApplicationState<TApp: App, TFactory> {
    New(TFactory),
    Initializing,
    Running(Box<Engine<TApp>>),
}

struct EngineApplication<TApp: App + 'static, TFactory> {
    attributes: WindowAttributes,
    state: ApplicationState<TApp, TFactory>,
    #[cfg(target_arch = "wasm32")]
    proxy: winit::event_loop::EventLoopProxy<Engine<TApp>>,
}

impl<TApp, TFactory> ApplicationHandler<Engine<TApp>> for EngineApplication<TApp, TFactory>
where
    TApp: App + 'static,
    TFactory: FnOnce(&WGPUContext) -> TApp + 'static,
{
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if !matches!(self.state, ApplicationState::New(_)) {
            return;
        }
        let ApplicationState::New(create_app) =
            std::mem::replace(&mut self.state, ApplicationState::Initializing)
        else {
            unreachable!();
        };

        let window = Arc::new(event_loop.create_window(self.attributes.clone()).unwrap());
        let display = event_loop.owned_display_handle();

        #[cfg(not(target_arch = "wasm32"))]
        self.user_event(
            event_loop,
            pollster::block_on(Engine::new(window, display, create_app)),
        );

        #[cfg(target_arch = "wasm32")]
        {
            let proxy = self.proxy.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let engine = Engine::new(window, display, create_app).await;
                let _ = proxy.send_event(engine);
            });
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, mut engine: Engine<TApp>) {
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
                let new_elapsed_seconds = engine.clock.elapsed().unwrap().as_secs_f32();
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
