use crate::core::wgpu_context::{DEPTH_FORMAT, WGPUContext};
use std::sync::Arc;
use wgpu::{Device, Surface, SurfaceCapabilities, SurfaceConfiguration};
use winit::dpi::PhysicalSize;
use winit::window::Window;

/// Presentation surface and its size-dependent attachments.
pub struct WindowSurface {
    pub surface: Surface<'static>,
    pub config: SurfaceConfiguration,
    pub(crate) depth_view: wgpu::TextureView,
}

impl WindowSurface {
    pub fn new(ctx: &WGPUContext, surface: Surface<'static>, size: PhysicalSize<u32>) -> Self {
        let config = surface_configuration(&surface.get_capabilities(&ctx.adapter), size);
        surface.configure(&ctx.device, &config);
        let depth_view = Self::create_depth_view(&ctx.device, &config);
        Self {
            surface,
            config,
            depth_view,
        }
    }

    pub fn resize(&mut self, device: &Device, new_size: PhysicalSize<u32>) {
        if new_size.width == 0 || new_size.height == 0 {
            return;
        }

        self.config.width = new_size.width;
        self.config.height = new_size.height;
        // Reconfigure even at the same size to recover outdated surfaces.
        self.surface.configure(device, &self.config);
        if self.depth_view.texture().width() != new_size.width
            || self.depth_view.texture().height() != new_size.height
        {
            self.depth_view = Self::create_depth_view(device, &self.config);
        }
    }

    pub fn recreate_surface(&mut self, ctx: &WGPUContext, window: Arc<Window>) {
        self.surface = ctx.instance.create_surface(window).unwrap();
        self.surface.configure(&ctx.device, &self.config);
    }

    fn create_depth_view(device: &Device, config: &SurfaceConfiguration) -> wgpu::TextureView {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        texture.create_view(&wgpu::TextureViewDescriptor::default())
    }
}

fn surface_configuration(
    capabilities: &SurfaceCapabilities,
    size: PhysicalSize<u32>,
) -> SurfaceConfiguration {
    // Prefer sRGB so the surface applies the output transfer function.
    let format = capabilities
        .formats
        .iter()
        .copied()
        .find(|f| f.is_srgb())
        .unwrap_or(capabilities.formats[0]);

    SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format,
        width: size.width.max(1),
        height: size.height.max(1),
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: capabilities.alpha_modes[0],
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
        color_space: wgpu::SurfaceColorSpace::Auto,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wgpu::{CompositeAlphaMode, TextureFormat};

    #[test]
    fn configuration_prefers_srgb_and_falls_back_to_the_preferred_surface_format() {
        for (formats, expected) in [
            (
                vec![TextureFormat::Bgra8Unorm, TextureFormat::Bgra8UnormSrgb],
                TextureFormat::Bgra8UnormSrgb,
            ),
            (
                vec![TextureFormat::Bgra8UnormSrgb, TextureFormat::Rgba8UnormSrgb],
                TextureFormat::Bgra8UnormSrgb,
            ),
            (
                vec![TextureFormat::Bgra8Unorm, TextureFormat::Rgba8Unorm],
                TextureFormat::Bgra8Unorm,
            ),
        ] {
            let capabilities = SurfaceCapabilities {
                formats,
                alpha_modes: vec![CompositeAlphaMode::Inherit, CompositeAlphaMode::Opaque],
                ..Default::default()
            };
            let config = surface_configuration(&capabilities, PhysicalSize::new(800, 600));
            assert_eq!(config.format, expected);
            assert_eq!(config.alpha_mode, CompositeAlphaMode::Inherit);
            assert_eq!((config.width, config.height), (800, 600));
        }
    }

    #[test]
    fn initial_configuration_clamps_each_zero_dimension() {
        let capabilities = SurfaceCapabilities {
            formats: vec![TextureFormat::Bgra8UnormSrgb],
            ..Default::default()
        };
        for (size, expected) in [
            (PhysicalSize::new(0, 0), (1, 1)),
            (PhysicalSize::new(0, 600), (1, 600)),
            (PhysicalSize::new(800, 0), (800, 1)),
        ] {
            let config = surface_configuration(&capabilities, size);
            assert_eq!((config.width, config.height), expected);
        }
    }
}
