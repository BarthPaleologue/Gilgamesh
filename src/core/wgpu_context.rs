use wgpu::{Adapter, Device, Instance, Queue, Surface};

/// GPU resources shared by rendering and compute, independent of a window.
pub struct WGPUContext {
    pub instance: Instance,
    pub adapter: Adapter,
    pub device: Device,
    pub queue: Queue,
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
        let options = wgpu::RequestAdapterOptions {
            compatible_surface,
            ..Default::default()
        };
        let adapter = instance.request_adapter(&options).await;
        #[cfg(target_arch = "wasm32")]
        let adapter = if compatible_surface.is_none() && adapter.is_err() {
            // WebGL selects its adapter from a canvas, even when no image is presented.
            let canvas = web_sys::OffscreenCanvas::new(1, 1).unwrap();
            let surface = instance
                .create_surface(wgpu::SurfaceTarget::OffscreenCanvas(canvas))
                .unwrap();
            instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    compatible_surface: Some(&surface),
                    ..options
                })
                .await
        } else {
            adapter
        };
        let adapter = adapter.unwrap();

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

        WGPUContext {
            instance,
            adapter,
            device,
            queue,
        }
    }
}
