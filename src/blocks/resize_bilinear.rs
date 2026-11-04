use crate::core::render_graph::{
    Binding, Image, ImageDescriptor, ImageSize, RenderGraph, fragment_pass,
};
use wgpu::{FilterMode, TextureSampleType};

/// Resizes a filterable color image with bilinear sampling and clamped edges.
/// Filtering happens in linear space, including for sRGB textures. Preserves RGBA and format.
/// `size` is relative to the frame target, as for other graph images.
pub fn resize_bilinear(source: Image, size: ImageSize, graph: &mut RenderGraph<'_>) -> Image {
    let descriptor = graph.image_descriptor(source);
    assert!(
        matches!(
            descriptor.format.sample_type(None, None),
            Some(TextureSampleType::Float { filterable: true })
        ),
        "resize_bilinear requires a filterable color format, got {:?}",
        descriptor.format
    );
    let output = graph.create_image("Bilinear Resize", ImageDescriptor { size, ..descriptor });
    fragment_pass(
        "Bilinear Resize",
        include_str!("resize_bilinear.wgsl"),
        [
            Binding::Texture(source),
            Binding::Sampler(wgpu::SamplerDescriptor {
                min_filter: FilterMode::Linear,
                mag_filter: FilterMode::Linear,
                ..Default::default()
            }),
            Binding::uniform_with(move |images| {
                let size = images.get(output).texture().size();
                [size.width, size.height, 0, 0]
            }),
        ],
        &[output],
        graph,
    );
    output
}
