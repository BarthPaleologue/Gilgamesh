use crate::core::render_graph::{Binding, Image, RenderGraph, fragment_pass};

pub(super) fn merge_levels(fine: Image, coarse: Image, graph: &mut RenderGraph<'_>) -> Image {
    let output = graph.create_image("Blur Merge", graph.image_descriptor(fine));
    fragment_pass(
        "Blur Merge",
        include_str!("merge_levels.wgsl"),
        [Binding::Texture(fine), Binding::Texture(coarse)],
        &[output],
        graph,
    );
    output
}
