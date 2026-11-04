use gilgamesh::core::render_graph::{Binding, Dispatch, Image, RenderGraph, compute_pass};

pub fn evolve(
    previous: Image,
    current: Image,
    step_due: bool,
    graph: &mut RenderGraph<'_>,
) -> Image {
    let current = graph.next_version(current);
    let dispatch = if step_due {
        Dispatch::for_image(current, [8, 8])
    } else {
        Dispatch::Skip
    };
    compute_pass(
        "Evolve Game of Life",
        include_str!("evolve.wgsl"),
        [Binding::Texture(previous), Binding::StorageTexture(current)],
        dispatch,
        graph,
    );
    current
}
