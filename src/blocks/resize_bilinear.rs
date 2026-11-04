use super::resize_pass::resize_pass;
use crate::core::render_graph::{Image, ImageSize, RenderGraph};

/// Resizes a filterable color image with bilinear sampling and clamped edges.
/// Filtering happens in linear space, including for sRGB textures. Preserves RGBA and format.
/// `size` is relative to the frame target, as for other graph images.
pub fn resize_bilinear(source: Image, size: ImageSize, graph: &mut RenderGraph<'_>) -> Image {
    resize_pass(
        "Bilinear Resize",
        include_str!("resize_bilinear.wgsl"),
        source,
        size,
        graph,
    )
}
