use crate::core::app::{App, FrameContext};
use crate::core::color_pass::ColorPass;
use crate::core::raster_renderer::RasterRenderer;
use crate::core::render_target::{OffscreenTarget, RenderTarget, create_color_view};
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

const SCENE_COLOR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
const LDR_IMAGE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

pub struct Engine<Application: App> {
    pub app: Application,
    pub window: Arc<Window>,
    pub wgpu_context: WGPUContext,
    pub window_surface: WindowSurface,

    renderer: RasterRenderer,
    tone_mapping: ColorPass,
    image_blit: ColorPass,
    targets: FrameTargets,
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
        let instance = wgpu::util::new_instance_with_webgpu_detection(
            wgpu::InstanceDescriptor::new_with_display_handle(Box::new(display)),
        )
        .await;
        let surface = instance.create_surface(window.clone()).unwrap();
        let wgpu_context = WGPUContext::new(instance, Some(&surface)).await;
        let window_surface = WindowSurface::new(&wgpu_context, surface, window.inner_size());
        assert!(
            wgpu_context
                .adapter
                .get_texture_format_features(SCENE_COLOR_FORMAT)
                .allowed_usages
                .contains(
                    wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING
                ),
            "HDR rendering requires Rgba16Float color attachments; WebGL needs floating-point color buffer support",
        );
        let tone_mapping = ColorPass::tone_mapping(&wgpu_context.device, LDR_IMAGE_FORMAT);
        let image_blit = ColorPass::image_blit(&wgpu_context.device, window_surface.config.format);
        let targets = FrameTargets::new(
            &wgpu_context.device,
            &tone_mapping,
            &image_blit,
            wgpu::Extent3d {
                width: window_surface.config.width,
                height: window_surface.config.height,
                depth_or_array_layers: 1,
            },
        );

        let renderer = RasterRenderer::new(&wgpu_context, SCENE_COLOR_FORMAT);
        let app = Application::new(&wgpu_context);

        Engine {
            app,
            renderer,
            tone_mapping,
            image_blit,
            targets,
            window,
            wgpu_context,
            window_surface,
            clock: Instant::now(),
            elapsed_seconds: 0.0,
            delta_seconds: 0.0,
            mouse: Mouse::new(),
        }
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.window_surface
            .resize(&self.wgpu_context.device, new_size);
    }

    fn manage_event(&mut self, event: &WindowEvent) {
        self.mouse.listen_to_event(event);
        self.app.on_window_event(event);
        if let WindowEvent::Resized(physical_size) = event {
            self.resize(*physical_size);
        }
    }

    fn render(&mut self, output: wgpu::SurfaceTexture) {
        let size = output.texture.size();
        if self.targets.scene.color.texture().size() != size {
            self.targets = FrameTargets::new(
                &self.wgpu_context.device,
                &self.tone_mapping,
                &self.image_blit,
                size,
            );
        }
        let color_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let frame = FrameContext {
            elapsed_seconds: self.elapsed_seconds,
            delta_seconds: self.delta_seconds,
            render_width: size.width,
            render_height: size.height,
            mouse: &self.mouse,
            wgpu_context: &self.wgpu_context,
        };
        self.app.update_before_render(&frame);
        self.renderer
            .update_frame_bindings(self.app.get_frame_bindings_data(), &self.wgpu_context);
        let mut encoder =
            self.wgpu_context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });
        self.renderer.encode(
            self.app.get_meshes(),
            &self.wgpu_context,
            &mut encoder,
            RenderTarget {
                color: &self.targets.scene.color,
                depth: &self.targets.scene.depth,
            },
        );
        self.tone_mapping.encode(
            &mut encoder,
            &self.targets.hdr_source,
            &self.targets.ldr_image,
        );
        self.image_blit
            .encode(&mut encoder, &self.targets.ldr_source, &color_view);
        self.wgpu_context
            .queue
            .submit(std::iter::once(encoder.finish()));
        self.mouse.reset();
        self.window.pre_present_notify();
        self.wgpu_context.queue.present(output);
    }
}

struct FrameTargets {
    scene: OffscreenTarget,
    ldr_image: wgpu::TextureView,
    hdr_source: wgpu::BindGroup,
    ldr_source: wgpu::BindGroup,
}

impl FrameTargets {
    fn new(
        device: &wgpu::Device,
        tone_mapping: &ColorPass,
        image_blit: &ColorPass,
        size: wgpu::Extent3d,
    ) -> Self {
        let scene = OffscreenTarget::new(device, size, SCENE_COLOR_FORMAT);
        let ldr_image = create_color_view(device, "LDR Image", size, LDR_IMAGE_FORMAT);
        let hdr_source = tone_mapping.bind_source(device, &scene.color);
        let ldr_source = image_blit.bind_source(device, &ldr_image);
        Self {
            scene,
            ldr_image,
            hdr_source,
            ldr_source,
        }
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
                match engine.window_surface.surface.get_current_texture() {
                    CurrentSurfaceTexture::Success(output) => engine.render(output),
                    CurrentSurfaceTexture::Suboptimal(output) => {
                        engine.render(output);
                        engine.resize(engine.window.inner_size());
                    }
                    CurrentSurfaceTexture::Outdated => {
                        engine.resize(engine.window.inner_size());
                    }
                    CurrentSurfaceTexture::Lost => {
                        engine
                            .window_surface
                            .recreate_surface(&engine.wgpu_context, engine.window.clone());
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
