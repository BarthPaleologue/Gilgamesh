#![cfg(not(target_arch = "wasm32"))]

use gilgamesh::blocks::overlay;
use gilgamesh::core::graph_executor::GraphExecutor;
use gilgamesh::core::render_graph::{
    Binding, Image, ImageDescriptor, ImageSize, RenderGraph, TextureFormat, fragment_pass,
};

fn solid(
    color: [f32; 4],
    format: TextureFormat,
    size: ImageSize,
    graph: &mut RenderGraph<'_>,
) -> Image {
    let image = graph.create_image("Solid", ImageDescriptor { format, size });
    fragment_pass(
        "Solid",
        "@group(0) @binding(0) var<uniform> color: vec4<f32>;\n\
         @fragment fn fs_main() -> @location(0) vec4<f32> { return color; }",
        [Binding::uniform(color)],
        &[image],
        graph,
    );
    image
}

#[test]
#[ignore = "GPU end-to-end check; run explicitly with approval"]
fn overlay_composites_premultiplied_alpha_in_linear_color_and_preserves_background_size() {
    pollster::block_on(async {
        let mut executor = GraphExecutor::new().await;
        let mut target = executor.create_target(3, 2);
        for (fg_alpha, bg_alpha, format, expected) in [
            (0.0, 1.0, TextureFormat::Rgba8Unorm, [0, 0, 255, 255]),
            (1.0, 1.0, TextureFormat::Rgba8Unorm, [255, 0, 0, 255]),
            (0.5, 0.5, TextureFormat::Rgba8Unorm, [128, 0, 64, 191]),
            (0.0, 0.0, TextureFormat::Rgba8Unorm, [0, 0, 0, 0]),
            (0.5, 1.0, TextureFormat::Rgba8UnormSrgb, [188, 0, 188, 255]),
        ] {
            let mut graph = RenderGraph::new();
            let foreground = solid(
                [fg_alpha, 0.0, 0.0, fg_alpha],
                TextureFormat::Rgba16Float,
                ImageSize::Fixed {
                    width: 1,
                    height: 1,
                },
                &mut graph,
            );
            let background = solid(
                [0.0, 0.0, bg_alpha, bg_alpha],
                format,
                ImageSize::Target,
                &mut graph,
            );
            let image = overlay(foreground, background, &mut graph);
            graph.export(image);
            executor.execute(graph, &mut target).unwrap();
            let image = target.read_image(executor.wgpu_context()).await.unwrap();
            assert_eq!(image.dimensions(), (3, 2));
            for pixel in image.pixels() {
                for (actual, expected) in pixel.0.into_iter().zip(expected) {
                    assert!(actual.abs_diff(expected) <= 1, "{pixel:?} != {expected}");
                }
            }
        }
    });
}
