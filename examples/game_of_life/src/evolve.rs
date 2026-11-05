use gilgamesh::core::render_graph::{Binding, Dispatch, Image, RenderGraph, compute_pass};

pub(crate) fn evolve(previous: Image, current: Image, graph: &mut RenderGraph<'_>) -> Image {
    let current = graph.next_version(current);
    compute_pass(
        "Evolve Game of Life",
        include_str!("evolve.wgsl"),
        [Binding::Texture(previous), Binding::StorageTexture(current)],
        Dispatch::for_image(current, [8, 8]),
        graph,
    );
    current
}
