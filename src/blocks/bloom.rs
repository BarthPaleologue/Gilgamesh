use super::pyramid_blur;
use crate::core::render_graph::{Binding, Image, RenderGraph, fragment_pass};

/// Threshold-free bloom in linear light, before tone mapping.
/// Mixes a normalized pyramidal blur with the source RGB by `weight` in [0, 1],
/// preserving source alpha, size and format. Use a filterable HDR format such as `Rgba16Float`.
/// Zero levels or zero weight returns the source unchanged.
pub fn bloom(source: Image, levels: u32, weight: f32, graph: &mut RenderGraph<'_>) -> Image {
    assert!(
        (0.0..=1.0).contains(&weight),
        "Bloom weight must be in [0, 1]"
    );
    if levels == 0 || weight == 0.0 {
        return source;
    }
    let blurred = pyramid_blur(source, levels, graph);
    let output = graph.create_image("Bloom Blend", graph.image_descriptor(source));
    fragment_pass(
        "Bloom Blend",
        include_str!("bloom.wgsl"),
        [
            Binding::Texture(source),
            Binding::Texture(blurred),
            Binding::uniform(weight),
        ],
        &[output],
        graph,
    );
    output
}
