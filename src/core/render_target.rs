/// Borrows single-sample color and depth attachments of matching dimensions.
/// Color uses the renderer's target format; depth uses `Depth32Float`.
pub struct RenderTarget<'a> {
    pub color: &'a wgpu::TextureView,
    pub depth: &'a wgpu::TextureView,
}

/// Owns persistent scene attachments, independent of a presentation surface.
#[must_use]
pub struct OffscreenTarget {
    pub color: wgpu::TextureView,
    pub depth: wgpu::TextureView,
}

impl OffscreenTarget {
    pub fn new(
        device: &wgpu::Device,
        size: wgpu::Extent3d,
        color_format: wgpu::TextureFormat,
    ) -> Self {
        Self {
            color: create_color_view(device, "Scene Color", size, color_format),
            depth: create_depth_view(device, size),
        }
    }
}

pub(crate) const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

pub(crate) fn create_color_view(
    device: &wgpu::Device,
    name: &str,
    size: wgpu::Extent3d,
    format: wgpu::TextureFormat,
) -> wgpu::TextureView {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(name),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    texture.create_view(&wgpu::TextureViewDescriptor::default())
}

pub(crate) fn create_depth_view(device: &wgpu::Device, size: wgpu::Extent3d) -> wgpu::TextureView {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Depth Texture"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: DEPTH_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    texture.create_view(&wgpu::TextureViewDescriptor::default())
}
