use super::{expose, rasterize, tone_map};
use crate::core::render_context::FrameBindingsData;
use crate::core::render_graph::{Image, RenderGraph, TextureFormat};
use crate::geometry::mesh::Mesh;

/// Renders a scene to an sRGB image using the camera's photographic exposure.
/// Composes rasterization in `Rgba32Float`, exposure in `Rgba16Float`, then tone mapping
/// to `Rgba8UnormSrgb`. `clear_color` is scene-linear RGBA, as in [`rasterize`].
/// Use the individual blocks to insert effects such as bloom before tone mapping.
pub fn render_scene<'a>(
    bindings: FrameBindingsData<'a>,
    meshes: impl IntoIterator<Item = &'a Mesh> + 'a,
    clear_color: [f32; 4],
    graph: &mut RenderGraph<'a>,
) -> Image {
    let exposure = bindings.exposure;
    let scene = rasterize(
        bindings,
        meshes,
        TextureFormat::Rgba32Float,
        clear_color,
        graph,
    );
    let exposed = expose(scene.color, exposure, TextureFormat::Rgba16Float, graph);
    tone_map(exposed, TextureFormat::Rgba8UnormSrgb, graph)
}
