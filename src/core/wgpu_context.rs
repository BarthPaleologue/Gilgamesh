use crate::core::object_bindings::ObjectBindings;
use crate::texture::Texture;
use std::rc::Rc;
use std::sync::Arc;
use wgpu::{Device, Queue, Surface, SurfaceConfiguration};
use winit::window::Window;

pub struct WGPUContext {
    pub surface: Surface<'static>,
    pub device: Device,
    pub queue: Queue,
    pub config: SurfaceConfiguration,
    empty_texture: Rc<Texture>,
    pub(crate) object_bind_group_layout: wgpu::BindGroupLayout,
    instance: wgpu::Instance,
}

impl WGPUContext {
    pub async fn new(
        window: Arc<Window>,
        display: winit::event_loop::OwnedDisplayHandle,
    ) -> WGPUContext {
        let size = window.inner_size();
        let instance = wgpu::util::new_instance_with_webgpu_detection(
            wgpu::InstanceDescriptor::new_with_display_handle(Box::new(display)),
        )
        .await;

        let surface = instance.create_surface(window).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
                ..Default::default()
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: if cfg!(target_arch = "wasm32") {
                    wgpu::Features::empty()
                } else {
                    wgpu::Features::POLYGON_MODE_LINE
                        | wgpu::Features::BUFFER_BINDING_ARRAY
                        | wgpu::Features::POLYGON_MODE_POINT
                        | wgpu::Features::ADDRESS_MODE_CLAMP_TO_BORDER
                },
                required_limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits())
                } else {
                    wgpu::Limits::default()
                },
                ..Default::default()
            })
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
            color_space: wgpu::SurfaceColorSpace::Auto,
        };
        surface.configure(&device, &config);

        let empty_texture = Rc::new(Texture::new_empty("Default Texture", &device));

        let object_bind_group_layout = ObjectBindings::create_layout(&device);

        WGPUContext {
            surface,
            device,
            queue,
            config,
            empty_texture,
            object_bind_group_layout,
            instance,
        }
    }

    pub fn empty_texture(&self) -> Rc<Texture> {
        self.empty_texture.clone()
    }

    pub fn recreate_surface(&mut self, window: Arc<Window>) {
        self.surface = self.instance.create_surface(window).unwrap();
        self.surface.configure(&self.device, &self.config);
    }
}
