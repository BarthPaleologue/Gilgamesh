use crate::core::render_graph::{
    Binding, Image, ImageDescriptor, ImageSize, RenderGraph, fragment_pass,
};
use wgpu::{FilterMode, TextureSampleType};

pub(super) fn resize_pass(
    label: &str,
    shader: &'static str,
    source: Image,
    size: ImageSize,
    graph: &mut RenderGraph<'_>,
) -> Image {
    let descriptor = graph.image_descriptor(source);
    assert!(
        matches!(
            descriptor.format.sample_type(None, None),
            Some(TextureSampleType::Float { filterable: true })
        ),
        "{label} requires a filterable color format, got {:?}",
        descriptor.format
    );
    let output = graph.create_image(label, ImageDescriptor { size, ..descriptor });
    fragment_pass(
        label,
        shader,
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
