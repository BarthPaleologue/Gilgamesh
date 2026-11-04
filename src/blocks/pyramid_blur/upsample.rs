use crate::blocks::resize_pass::resize_pass;
use crate::core::render_graph::{Image, ImageSize, RenderGraph};

pub(super) fn upsample(source: Image, size: ImageSize, graph: &mut RenderGraph<'_>) -> Image {
    resize_pass(
        "Pyramid Upsample",
        include_str!("upsample.wgsl"),
        source,
        size,
        graph,
    )
}
