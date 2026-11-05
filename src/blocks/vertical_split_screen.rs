use crate::core::render_graph::{Binding, Image, RenderGraph, fragment_pass};
use wgpu::TextureSampleType;

/// Combines color images across a vertical divider, preserving the left image's size and format.
/// `split` is a fraction in [0, 1] measured from the left. The right image is resampled
/// with nearest-neighbor lookup when its dimensions differ. HDR stays HDR.
///
/// # Panics
/// Panics if `split` is outside [0, 1] or NaN, either image belongs to another graph,
/// or either format is not a float-sampled color format.
pub fn vertical_split_screen(
    left: Image,
    right: Image,
    split: f32,
    graph: &mut RenderGraph<'_>,
) -> Image {
    assert!(
        (0.0..=1.0).contains(&split),
        "Split position must be in [0, 1]"
    );
    let descriptor = graph.image_descriptor(left);
    for image in [left, right] {
        assert!(
            matches!(
                graph.image_descriptor(image).format.sample_type(None, None),
                Some(TextureSampleType::Float { .. })
            ),
            "vertical_split_screen requires float-sampled color images"
        );
    }
    let output = graph.create_image("Vertical Split Screen", descriptor);
    fragment_pass(
        "Vertical Split Screen",
        include_str!("vertical_split_screen.wgsl"),
        [
            Binding::Texture(left),
            Binding::Texture(right),
            Binding::uniform(split),
        ],
        &[output],
        graph,
    );
    output
}
