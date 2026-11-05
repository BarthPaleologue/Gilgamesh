use crate::camera::Exposure;
use crate::core::render_graph::{Binding, Image, ImageDescriptor, RenderGraph, fragment_pass};
use wgpu::TextureFormat;

/// Applies photographic exposure to scene-linear RGB, preserving alpha and dimensions.
/// Use an `Rgba32Float` source to retain raw luminance above the half-float range,
/// and an `Rgba16Float` output for filtering, bloom and tone mapping.
pub fn expose(
    source: Image,
    exposure: Exposure,
    format: TextureFormat,
    graph: &mut RenderGraph<'_>,
) -> Image {
    let output = graph.create_image(
        "Exposed Image",
        ImageDescriptor {
            format,
            ..graph.image_descriptor(source)
        },
    );
    fragment_pass(
        "Exposure",
        include_str!("expose.wgsl"),
        [Binding::Texture(source), Binding::uniform(exposure.gain())],
        &[output],
        graph,
    );
    output
}
