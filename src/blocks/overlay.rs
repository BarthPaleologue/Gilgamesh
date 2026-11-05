use crate::core::render_graph::{Binding, Image, RenderGraph, fragment_pass};
use wgpu::TextureSampleType;

/// Composites premultiplied-alpha RGBA in linear color, preserving the background's size and format.
/// RGB must already be multiplied by alpha. Zero-alpha RGB is preserved as additive light.
/// The foreground is resampled with nearest-neighbor lookup if dimensions differ.
/// Place display UI after exposure and tone mapping so camera settings do not affect it.
///
/// # Panics
/// Panics if an image belongs to another graph or is not a float-sampled color image.
pub fn overlay(foreground: Image, background: Image, graph: &mut RenderGraph<'_>) -> Image {
    for image in [foreground, background] {
        assert!(
            matches!(
                graph.image_descriptor(image).format.sample_type(None, None),
                Some(TextureSampleType::Float { .. })
            ),
            "overlay requires float-sampled color images"
        );
    }
    let output = graph.create_image("Overlay", graph.image_descriptor(background));
    fragment_pass(
        "Overlay",
        include_str!("overlay.wgsl"),
        [Binding::Texture(foreground), Binding::Texture(background)],
        &[output],
        graph,
    );
    output
}
