#![cfg(not(target_arch = "wasm32"))]

use gilgamesh::blocks::vertical_split_screen;
use gilgamesh::core::graph_executor::GraphExecutor;
use gilgamesh::core::render_graph::{
    Binding, Dispatch, ImageDescriptor, ImageSize, RenderGraph, compute_pass, fragment_pass,
};
use wgpu::TextureFormat;

const MRT: &str = r"
struct Outputs {
    @location(0) left: vec4f,
    @location(1) right: vec4f,
}
@fragment fn fs_main() -> Outputs {
    return Outputs(vec4f(1, 0, 0, 1), vec4f(0, 1, 0, 1));
}
";
const COMPUTE: &str = r"
@group(0) @binding(0) var left: texture_2d<f32>;
@group(0) @binding(1) var right: texture_2d<f32>;
@group(0) @binding(2) var first: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(3) var second: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(4) var<uniform> swap: vec4u;
@compute @workgroup_size(8, 8) fn main(@builtin(global_invocation_id) id: vec3u) {
    if any(id.xy >= textureDimensions(first)) { return; }
    let p = vec2i(id.xy);
    let a = textureLoad(left, p, 0);
    let b = textureLoad(right, p, 0);
    textureStore(first, p, select(a, b, swap.x != 0u));
    textureStore(second, p, select(b, a, swap.x != 0u));
}
";

#[test]
#[ignore = "GPU end-to-end check; run explicitly with approval"]
fn helpers_support_mrt_multiple_compute_outputs_and_changing_cached_parameters() {
    pollster::block_on(async {
        let mut executor = GraphExecutor::new().await;
        let mut target = executor.create_target(19, 11);
        for (width, height, swap, reverse_shader) in [
            (19, 11, 0_u32, false),
            (19, 11, 1, false),
            (7, 3, 0, false),
            (7, 3, 0, true),
            (19, 11, 1, true),
        ] {
            executor.resize_target(&mut target, width, height);
            let mut graph = RenderGraph::new();
            let descriptor = ImageDescriptor {
                size: ImageSize::Target,
                format: TextureFormat::Rgba8Unorm,
            };
            let left = graph.create_image("MRT left", descriptor);
            let right = graph.create_image("MRT right", descriptor);
            let shader = if reverse_shader {
                MRT.replace("vec4f(1, 0, 0, 1)", "vec4f(0, 0, 1, 1)")
            } else {
                MRT.to_owned()
            };
            fragment_pass("MRT", shader, [], &[left, right], &mut graph);
            let first = graph.create_image("Compute first", descriptor);
            let second = graph.create_image("Compute second", descriptor);
            compute_pass(
                "Combine",
                COMPUTE,
                [
                    Binding::Texture(left),
                    Binding::Texture(right),
                    Binding::StorageTexture(first),
                    Binding::StorageTexture(second),
                    Binding::uniform(swap),
                ],
                Dispatch::for_image(first, [8, 8]),
                &mut graph,
            );
            let output = vertical_split_screen(first, second, 0.5, &mut graph);
            graph.export(output);
            executor.execute(graph, &mut target).unwrap();
            let image = target.read_image(executor.wgpu_context()).await.unwrap();
            for (x, _, pixel) in image.enumerate_pixels() {
                let green = (x as f32 + 0.5 >= width as f32 * 0.5) ^ (swap != 0);
                let expected = if green {
                    [0, 255, 0, 255]
                } else if reverse_shader {
                    [0, 0, 255, 255]
                } else {
                    [255, 0, 0, 255]
                };
                assert_eq!(pixel.0, expected);
            }
        }
    });
}
