#![cfg(not(target_arch = "wasm32"))]

use gilgamesh::blocks::{tone_map, vertical_split_screen, visualize_depth};
use gilgamesh::core::graph_executor::GraphExecutor;
use gilgamesh::core::render_graph::{ImageDescriptor, ImageSize, RenderGraph};
use gilgamesh::core::wgpu_context::WGPUContext;
use wgpu::util::{DeviceExt, TextureDataOrder};
use wgpu::{TextureFormat as Format, TextureUsages as Usage};

#[test]
#[ignore = "GPU end-to-end check; run explicitly with approval"]
fn visualize_depth_shows_raw_depth_and_vertical_split_preserves_hdr() {
    pollster::block_on(async {
        let mut executor = GraphExecutor::new().await;
        let mut target = executor.create_target(3, 2);
        let mut graph = RenderGraph::new();
        let depth = graph.create_image(
            "Depth",
            ImageDescriptor {
                size: ImageSize::Target,
                format: Format::Depth32Float,
            },
        );
        graph.add_pass(
            "Clear Depth",
            &[],
            &[(depth, Usage::RENDER_ATTACHMENT)],
            move |context| {
                let _pass = context
                    .encoder
                    .begin_render_pass(&wgpu::RenderPassDescriptor {
                        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                            view: context.images.get(depth),
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Clear(0.5),
                                store: wgpu::StoreOp::Store,
                            }),
                            stencil_ops: None,
                        }),
                        ..Default::default()
                    });
            },
        );
        let output = visualize_depth(depth, &mut graph);
        graph.export(output);
        executor.execute(graph, &mut target).unwrap();
        for pixel in target
            .read_image(executor.wgpu_context())
            .await
            .unwrap()
            .pixels()
        {
            assert_pixel(pixel.0, [188, 188, 188, 255]);
        }

        let left = upload(
            executor.wgpu_context(),
            Format::Rgba32Float,
            (5, 2),
            bytemuck::cast_slice(&[8.0_f32, 0.0, 1.0, 1.0].repeat(10)),
        );
        let right = upload(
            executor.wgpu_context(),
            Format::Rgba32Float,
            (2, 1),
            bytemuck::cast_slice(&[4.0_f32, 1.0, 0.25, 1.0].repeat(2)),
        );
        executor.resize_target(&mut target, 5, 2);
        for split in [0.0, 0.4, 1.0] {
            let mut graph = RenderGraph::new();
            let a = graph.import_texture("Left HDR", &left);
            let b = graph.import_texture("Right HDR", &right);
            let comparison = vertical_split_screen(a, b, split, &mut graph);
            assert_eq!(
                graph.image_descriptor(comparison).format,
                Format::Rgba32Float
            );
            let output = tone_map(comparison, Format::Rgba8UnormSrgb, &mut graph);
            graph.export(output);
            executor.execute(graph, &mut target).unwrap();
            let pixels = target.read_image(executor.wgpu_context()).await.unwrap();
            for (x, _, pixel) in pixels.enumerate_pixels() {
                let expected = if (x as f32 + 0.5) < split * 5.0 {
                    [254, 189, 199, 255]
                } else {
                    [253, 182, 158, 255]
                };
                assert_pixel(pixel.0, expected);
            }
        }
    });
}

fn upload(
    gpu: &WGPUContext,
    format: Format,
    (width, height): (u32, u32),
    bytes: &[u8],
) -> wgpu::Texture {
    gpu.device.create_texture_with_data(
        &gpu.queue,
        &wgpu::TextureDescriptor {
            label: Some("Block Test Input"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: Usage::TEXTURE_BINDING,
            view_formats: &[],
        },
        TextureDataOrder::LayerMajor,
        bytes,
    )
}

fn assert_pixel(actual: [u8; 4], expected: [u8; 4]) {
    for (actual, expected) in actual.into_iter().zip(expected) {
        assert!(actual.abs_diff(expected) <= 1, "{actual} != {expected}");
    }
}
