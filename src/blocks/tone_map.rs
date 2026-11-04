use crate::core::render_graph::{Binding, Image, ImageDescriptor, RenderGraph, fragment_pass};
use wgpu::TextureFormat;

/// Applies Khronos PBR Neutral to linear HDR. Choose an sRGB output format for PNG captures.
pub fn tone_map(source: Image, format: TextureFormat, graph: &mut RenderGraph<'_>) -> Image {
    let output = graph.create_image(
        "Tone Mapped Image",
        ImageDescriptor {
            format,
            ..graph.image_descriptor(source)
        },
    );
    fragment_pass(
        "Tone Mapping",
        include_str!("tone_map.wgsl"),
        [Binding::Texture(source)],
        &[output],
        graph,
    );
    output
}
