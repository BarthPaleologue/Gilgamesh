use crate::core::scene::Scene;
use crate::core::wgpu_context::WGPUContext;
use cgmath::*;
use std::sync::Arc;
use std::time::SystemTime;
use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalSize, Size};
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowAttributes, WindowId};

pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.5, 1.0,
);

pub struct Engine {
    pub window: Arc<Window>,
    pub wgpu_context: WGPUContext,

    clock: SystemTime,
    elapsed_time: f32,
    delta_time: f32,

    pub on_window_resize: Vec<Box<dyn FnMut(PhysicalSize<u32>)>>,
}

impl Engine {
    pub fn run(name: &str, width: u32, height: u32, create_scene: fn(&Engine) -> Scene) {
        cfg_if::cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                std::panic::set_hook(Box::new(console_error_panic_hook::hook));
                console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
            } else {
                env_logger::init();
            }
        }

        let event_loop = EventLoop::<Engine>::with_user_event().build().unwrap();
        let attributes = Window::default_attributes()
            .with_inner_size(Size::Physical(PhysicalSize { width, height }))
            .with_title(name);
        let application = EngineApplication {
            attributes,
            create_scene,
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

        Engine {
            window,
            wgpu_context,
            clock: SystemTime::now(),
            elapsed_time: 0.0,
            delta_time: 0.0,
            on_window_resize: Vec::new(),
        }
    }

    pub fn get_delta_time(&self) -> f32 {
        self.delta_time
    }

    pub fn get_elapsed_time(&self) -> f32 {
        self.elapsed_time
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

    pub fn manage_event(&mut self, event: &WindowEvent) {
        if let WindowEvent::Resized(physical_size) = event {
            self.resize(*physical_size);
        }
    }
}

enum ApplicationState {
    New,
    Initializing,
    Running { engine: Engine, scene: Scene },
}

struct EngineApplication {
    attributes: WindowAttributes,
    create_scene: fn(&Engine) -> Scene,
    state: ApplicationState,
    #[cfg(target_arch = "wasm32")]
    proxy: winit::event_loop::EventLoopProxy<Engine>,
}

impl ApplicationHandler<Engine> for EngineApplication {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if !matches!(self.state, ApplicationState::New) {
            return;
        }

        let window = Arc::new(event_loop.create_window(self.attributes.clone()).unwrap());
        let display = event_loop.owned_display_handle();
        self.state = ApplicationState::Initializing;

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

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, mut engine: Engine) {
        let mut scene = (self.create_scene)(&engine);
        let size = engine.window.inner_size();
        engine.resize(size);
        scene.resize(size);
        engine.window.request_redraw();
        self.state = ApplicationState::Running { engine, scene };
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

        let ApplicationState::Running { engine, scene } = &mut self.state else {
            return;
        };
        if window_id != engine.window.id() {
            return;
        }

        engine.manage_event(&event);
        scene.manage_event(&event, engine);

        match event {
            WindowEvent::KeyboardInput { event, .. }
                if event.state == ElementState::Pressed
                    && event.physical_key == PhysicalKey::Code(KeyCode::Escape) =>
            {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                let new_elapsed_time = engine.clock.elapsed().unwrap().as_secs_f32();
                engine.delta_time = new_elapsed_time - engine.elapsed_time;
                engine.elapsed_time = new_elapsed_time;

                use wgpu::CurrentSurfaceTexture;
                match engine.wgpu_context.surface.get_current_texture() {
                    CurrentSurfaceTexture::Success(output) => scene.render(engine, output),
                    CurrentSurfaceTexture::Suboptimal(output) => {
                        scene.render(engine, output);
                        engine.resize(engine.window.inner_size());
                    }
                    CurrentSurfaceTexture::Outdated => {
                        let size = engine.window.inner_size();
                        scene.resize(size);
                        engine.resize(size);
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
