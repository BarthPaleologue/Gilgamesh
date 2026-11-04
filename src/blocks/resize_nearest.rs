use crate::core::render_graph::{
    Binding, Image, ImageDescriptor, ImageSize, RenderGraph, fragment_pass,
};
use wgpu::TextureSampleType;

/// Resizes a float-sampled color image using nearest-neighbor sampling.
/// Preserves its channels, format and color space, including HDR values.
/// The source format must support render attachments.
/// `size` is resolved relative to the frame target, as for other graph images.
pub fn resize_nearest(source: Image, size: ImageSize, graph: &mut RenderGraph<'_>) -> Image {
    let descriptor = graph.image_descriptor(source);
    assert!(
        matches!(
            descriptor.format.sample_type(None, None),
            Some(TextureSampleType::Float { .. })
        ),
        "resize_nearest requires a float-sampled color format, got {:?}",
        descriptor.format
    );
    let output = graph.create_image("Nearest Resize", ImageDescriptor { size, ..descriptor });
    fragment_pass(
        "Nearest Resize",
        include_str!("resize_nearest.wgsl"),
        [
            Binding::Texture(source),
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
