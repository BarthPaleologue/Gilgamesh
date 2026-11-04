use crate::core::object_bindings::ObjectBindings;
use crate::texture::{Linear, Texture};
use wgpu::{Adapter, Device, Instance, Queue, Surface};

/// GPU resources shared by rendering and compute, independent of a window.
pub struct WGPUContext {
    pub instance: Instance,
    pub adapter: Adapter,
    pub device: Device,
    pub queue: Queue,
    fallback_texture: Texture<Linear>,
    pub(crate) object_bind_group_layout: wgpu::BindGroupLayout,
}

impl WGPUContext {
    /// Selects an adapter, optionally constrained to a presentation surface.
    /// Pass `None` to initialize GPU resources without a window or surface.
    ///
    /// Headless initialization on desktop:
    /// ```no_run
    /// use gilgamesh::core::wgpu_context::WGPUContext;
    ///
    /// let gpu = pollster::block_on(WGPUContext::new(wgpu::Instance::default(), None));
    /// ```
    pub async fn new(instance: Instance, compatible_surface: Option<&Surface<'_>>) -> WGPUContext {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface,
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

        let fallback_texture = Texture::<Linear>::new_empty("Fallback Texture", &device);

        let object_bind_group_layout = ObjectBindings::create_layout(&device);

        WGPUContext {
            instance,
            adapter,
            device,
            queue,
            fallback_texture,
            object_bind_group_layout,
        }
    }

    pub fn fallback_texture(&self) -> &Texture<Linear> {
        &self.fallback_texture
    }
}
