use crate::blocks::resize_pass::resize_pass;
use crate::core::render_graph::{Image, ImageSize, RenderGraph};

pub(super) fn downsample(source: Image, size: ImageSize, graph: &mut RenderGraph<'_>) -> Image {
    resize_pass(
        "Pyramid Downsample",
        include_str!("downsample.wgsl"),
        source,
        size,
        graph,
    )
}
