use crate::core::render_graph::{Binding, Image, ImageDescriptor, RenderGraph, fragment_pass};
use wgpu::{TextureFormat, TextureSampleType};

/// Displays raw depth as grayscale in an sRGB image, without linearization.
pub fn visualize_depth(source: Image, graph: &mut RenderGraph<'_>) -> Image {
    let descriptor = graph.image_descriptor(source);
    assert_eq!(
        descriptor.format.sample_type(None, None),
        Some(TextureSampleType::Depth),
        "visualize_depth requires a depth format, got {:?}",
        descriptor.format
    );
    let output = graph.create_image(
        "Visualize Depth",
        ImageDescriptor {
            format: TextureFormat::Rgba8UnormSrgb,
            ..descriptor
        },
    );
    fragment_pass(
        "Visualize Depth",
        include_str!("visualize_depth.wgsl"),
        [
            Binding::Texture(source),
            Binding::Sampler(wgpu::SamplerDescriptor::default()),
        ],
        &[output],
        graph,
    );
    output
}
