//! Built-in blocks. Applications can compose these or register their own GPU passes.

use crate::core::raster_renderer::RasterRenderer;
use crate::core::render_context::FrameBindingsData;
use crate::core::render_graph::{Image, ImageDescriptor, ImageSize, RenderGraph};
use crate::core::render_target::{DEPTH_FORMAT, RenderTarget};
use crate::geometry::mesh::Mesh;
use wgpu::{TextureFormat, TextureUsages};

pub struct RasterOutput {
    pub color: Image,
    pub depth: Image,
}

/// Rasterizes scene data into scene-linear luminance and reverse depth.
/// Use `Rgba32Float` for physical luminance, then apply [`super::expose`] before bloom and tone mapping.
pub fn rasterize<'a>(
    bindings: FrameBindingsData<'a>,
    meshes: impl IntoIterator<Item = &'a Mesh> + 'a,
    color_format: TextureFormat,
    graph: &mut RenderGraph<'a>,
) -> RasterOutput {
    let color = graph.create_image(
        "Raster Color",
        ImageDescriptor {
            size: ImageSize::Target,
            format: color_format,
        },
    );
    let depth = graph.create_image(
        "Raster Depth",
        ImageDescriptor {
            size: ImageSize::Target,
            format: DEPTH_FORMAT,
        },
    );
    graph.add_pass(
        "Rasterize",
        &[],
        &[
            (color, TextureUsages::RENDER_ATTACHMENT),
            (depth, TextureUsages::RENDER_ATTACHMENT),
        ],
        move |context| {
            let renderer = context.cache.get_or_insert_with(color_format, || {
                RasterRenderer::new(context.gpu, color_format)
            });
            let size = context.images.get(color).texture().size();
            let aspect_ratio = f64::from(size.width) / f64::from(size.height);
            renderer.update_frame_bindings(&bindings, aspect_ratio, context.gpu);
            renderer.encode(
                meshes,
                context.gpu,
                context.encoder,
                &RenderTarget {
                    color: context.images.get(color),
                    depth: context.images.get(depth),
                },
            );
        },
    );
    RasterOutput { color, depth }
}
