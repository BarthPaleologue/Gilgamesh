use gilgamesh::core::render_graph::{Binding, Dispatch, Image, RenderGraph, compute_pass};

pub fn randomize(target: Image, seed: u32, graph: &mut RenderGraph<'_>) -> Image {
    let target = graph.next_version(target);
    compute_pass(
        "Randomize Game of Life",
        include_str!("randomize.wgsl"),
        [Binding::StorageTexture(target), Binding::uniform(seed)],
        Dispatch::for_image(target, [8, 8]),
        graph,
    );
    target
}
