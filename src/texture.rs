use crate::core::wgpu_context::WGPUContext;
use image::GenericImageView;
use std::marker::PhantomData;
use std::rc::Rc;

/// Embeds an image file at compile time and creates a texture at runtime.
///
/// Use `embed_texture!(name, path, encoding, context)`, with a literal path
/// relative to the calling source file and an encoding of [`Srgb`] or [`Linear`].
/// Returns a [`Texture`] without wrapping it in an [`Rc`].
///
/// # Panics
/// Panics if the embedded bytes cannot be decoded as a supported image.
#[macro_export]
macro_rules! embed_texture {
    ($name:expr, $path:literal, $encoding:ty, $context:expr $(,)?) => {
        $crate::texture::Texture::<$encoding>::new($name, ::core::include_bytes!($path), $context)
    };
}

/// Color images decoded from sRGB when sampled.
pub struct Srgb;

/// Linear colors or data sampled without an sRGB conversion.
pub struct Linear;

mod sealed {
    pub trait Sealed {}
}

/// Encodings supported by material textures.
///
/// ```compile_fail,E0277
/// use gilgamesh::texture::ColorEncoding;
///
/// struct UncheckedEncoding;
/// impl ColorEncoding for UncheckedEncoding {
///     const RGBA8_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
/// }
/// ```
pub trait ColorEncoding: sealed::Sealed + 'static {
    const RGBA8_FORMAT: wgpu::TextureFormat;
}

impl sealed::Sealed for Srgb {}
impl sealed::Sealed for Linear {}

impl ColorEncoding for Srgb {
    const RGBA8_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
}

impl ColorEncoding for Linear {
    const RGBA8_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
}

/// A sampled texture whose encoding is selected at construction.
/// Color inputs accept either encoding; data inputs require [`Linear`].
///
/// ```no_run
/// use gilgamesh::materials::PbrMaterial;
/// use gilgamesh::texture::{Linear, Srgb, Texture};
/// use std::rc::Rc;
///
/// fn bind(
///     material: &mut PbrMaterial,
///     srgb: Rc<Texture<Srgb>>,
///     linear: Rc<Texture<Linear>>,
/// ) {
///     material.set_albedo_texture(srgb.clone());
///     material.set_albedo_texture(linear.clone());
///     material.set_emissive_luminance(gilgamesh::photometry::Nits::new(100.0));
///     material.set_emissive_color(1.0, 1.0, 1.0);
///     material.set_emissive_texture(srgb.clone());
///     material.set_emissive_texture(linear.clone());
///     material.set_normal_map(linear);
/// }
/// ```
#[must_use]
pub struct Texture<Encoding: ColorEncoding> {
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    encoding: PhantomData<Encoding>,
}

/// A color input whose storage can be either sRGB or linear.
#[derive(Clone)]
pub enum ColorTexture {
    Srgb(Rc<Texture<Srgb>>),
    Linear(Rc<Texture<Linear>>),
}

impl From<Rc<Texture<Srgb>>> for ColorTexture {
    fn from(texture: Rc<Texture<Srgb>>) -> Self {
        Self::Srgb(texture)
    }
}

impl From<Rc<Texture<Linear>>> for ColorTexture {
    fn from(texture: Rc<Texture<Linear>>) -> Self {
        Self::Linear(texture)
    }
}

/// Supplies GPU bindings independently of the texture's storage encoding.
pub(crate) trait SampledTexture {
    fn create_bind_group_entries(&self, start_index: u32) -> [wgpu::BindGroupEntry<'_>; 2];
}

impl<Encoding: ColorEncoding> Texture<Encoding> {
    pub fn new_empty(name: &str, device: &wgpu::Device) -> Self {
        let texture_size = wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            // All textures are stored as 3D, we represent our 2D texture
            // by setting depth to 1.
            size: texture_size,
            mip_level_count: 1, // We'll talk about this a little later
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Encoding::RGBA8_FORMAT,
            // TEXTURE_BINDING tells wgpu that we want to use this texture in shaders
            // COPY_DST means that we want to copy data to this texture
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some(name),
            view_formats: &[],
        });

        // We don't need to configure the texture view much, so let's
        // let wgpu define it.
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        Texture {
            view,
            sampler,
            encoding: PhantomData,
        }
    }

    /// # Panics
    /// Panics if `bytes` cannot be decoded as a supported image.
    pub fn new(name: &str, bytes: &[u8], wgpu_context: &WGPUContext) -> Self {
        let image = image::load_from_memory(bytes).expect("Failed to decode material texture");
        let image_rgba = image.to_rgba8();

        let dimensions = image.dimensions();
        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let texture = wgpu_context
            .device
            .create_texture(&wgpu::TextureDescriptor {
                // All textures are stored as 3D, we represent our 2D texture
                // by setting depth to 1.
                size: texture_size,
                mip_level_count: 1, // We'll talk about this a little later
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: Encoding::RGBA8_FORMAT,
                // TEXTURE_BINDING tells wgpu that we want to use this texture in shaders
                // COPY_DST means that we want to copy data to this texture
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                label: Some(name),
                view_formats: &[],
            });

        wgpu_context.queue.write_texture(
            // Tells wgpu where to copy the pixel data
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            // The actual pixel data
            &image_rgba,
            // The layout of the texture
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            texture_size,
        );

        // We don't need to configure the texture view much, so let's
        // let wgpu define it.
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = wgpu_context
            .device
            .create_sampler(&wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::MipmapFilterMode::Nearest,
                ..Default::default()
            });

        Texture {
            view,
            sampler,
            encoding: PhantomData,
        }
    }
}

pub(crate) fn bind_group_layout_entries(start_index: u32) -> [wgpu::BindGroupLayoutEntry; 2] {
    [
        wgpu::BindGroupLayoutEntry {
            binding: start_index,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                multisampled: false,
                view_dimension: wgpu::TextureViewDimension::D2,
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
            },
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: start_index + 1,
            visibility: wgpu::ShaderStages::FRAGMENT,
            // This should match the filterable field of the
            // corresponding Texture entry above.
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        },
    ]
}

impl<Encoding: ColorEncoding> SampledTexture for Texture<Encoding> {
    fn create_bind_group_entries(&self, start_index: u32) -> [wgpu::BindGroupEntry<'_>; 2] {
        [
            wgpu::BindGroupEntry {
                binding: start_index,
                resource: wgpu::BindingResource::TextureView(&self.view),
            },
            wgpu::BindGroupEntry {
                binding: start_index + 1,
                resource: wgpu::BindingResource::Sampler(&self.sampler),
            },
        ]
    }
}

impl SampledTexture for ColorTexture {
    fn create_bind_group_entries(&self, start_index: u32) -> [wgpu::BindGroupEntry<'_>; 2] {
        match self {
            Self::Srgb(texture) => texture.create_bind_group_entries(start_index),
            Self::Linear(texture) => texture.create_bind_group_entries(start_index),
        }
    }
}
