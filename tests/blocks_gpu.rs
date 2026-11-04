#![cfg(not(target_arch = "wasm32"))]

use gilgamesh::blocks::{resize_nearest, tone_map, vertical_split_screen, visualize_depth};
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

#[test]
#[ignore = "GPU end-to-end check; run explicitly with approval"]
fn nearest_resize_preserves_rgba_srgb_and_hdr_across_sizes() {
    pollster::block_on(async {
        let mut executor = GraphExecutor::new().await;
        let rgba = [
            [8_u8, 31, 200, 17],
            [220, 7, 80, 49],
            [55, 128, 14, 96],
            [1, 90, 180, 128],
            [250, 50, 100, 200],
            [100, 220, 40, 255],
        ];
        let hdr = [
            [8_f32, 0.25, 1.0, 0.125],
            [0.25, 4.0, 1.0, 0.25],
            [1.0, 0.5, 16.0, 0.375],
            [4.0, 8.0, 0.0, 0.5],
            [0.0, 2.0, 0.5, 0.75],
            [16.0, 0.0, 8.0, 1.0],
        ];
        let inputs = [
            upload(
                executor.wgpu_context(),
                Format::Rgba8Unorm,
                (3, 2),
                bytemuck::cast_slice(&rgba),
            ),
            upload(
                executor.wgpu_context(),
                Format::Rgba8UnormSrgb,
                (3, 2),
                bytemuck::cast_slice(&rgba),
            ),
            upload(
                executor.wgpu_context(),
                Format::Rgba32Float,
                (3, 2),
                bytemuck::cast_slice(&hdr),
            ),
        ];
        for input in inputs {
            let mut target = executor.create_target(3, 2);
            let baseline = if input.format() == Format::Rgba32Float {
                let mut graph = RenderGraph::new();
                let source = graph.import_texture("HDR reference", &input);
                let output = tone_map(source, Format::Rgba8UnormSrgb, &mut graph);
                graph.export(output);
                executor.execute(graph, &mut target).unwrap();
                target.read_image(executor.wgpu_context()).await.unwrap()
            } else {
                image::RgbaImage::from_raw(3, 2, bytemuck::cast_slice(&rgba).to_vec()).unwrap()
            };
            for (width, height, size) in [
                (7, 5, ImageSize::Target),
                (
                    2,
                    1,
                    ImageSize::Fixed {
                        width: 2,
                        height: 1,
                    },
                ),
                (1, 1, ImageSize::Target),
                (
                    5,
                    3,
                    ImageSize::Fixed {
                        width: 5,
                        height: 3,
                    },
                ),
                (3, 2, ImageSize::Target),
            ] {
                executor.resize_target(&mut target, width, height);
                let mut graph = RenderGraph::new();
                let source = graph.import_texture("Color grid", &input);
                let resized = resize_nearest(source, size, &mut graph);
                assert_eq!(graph.image_descriptor(resized).format, input.format());
                let output = if input.format() == Format::Rgba32Float {
                    tone_map(resized, Format::Rgba8UnormSrgb, &mut graph)
                } else {
                    resized
                };
                graph.export(output);
                executor.execute(graph, &mut target).unwrap();
                let pixels = target.read_image(executor.wgpu_context()).await.unwrap();
                assert_eq!(pixels.dimensions(), (width, height));
                for (x, y, actual) in pixels.enumerate_pixels() {
                    let sx = (2 * x + 1) * 3 / (2 * width);
                    let sy = (2 * y + 1) * 2 / (2 * height);
                    assert_pixel(actual.0, baseline.get_pixel(sx, sy).0);
                }
            }

            executor.resize_target(&mut target, 4, 2);
            let mut graph = RenderGraph::new();
            let source = graph.import_texture("Color grid", &input);
            let half = resize_nearest(
                source,
                ImageSize::Scaled {
                    numerator: 1,
                    denominator: 2,
                },
                &mut graph,
            );
            let resized = resize_nearest(half, ImageSize::Target, &mut graph);
            let output = if input.format() == Format::Rgba32Float {
                tone_map(resized, Format::Rgba8UnormSrgb, &mut graph)
            } else {
                resized
            };
            graph.export(output);
            executor.execute(graph, &mut target).unwrap();
            let pixels = target.read_image(executor.wgpu_context()).await.unwrap();
            for (x, _, actual) in pixels.enumerate_pixels() {
                let sx = if x < 2 { 0 } else { 2 };
                assert_pixel(actual.0, baseline.get_pixel(sx, 1).0);
            }

            executor.resize_target(&mut target, 3, 2);
            for use_split in [true, false] {
                let mut graph = RenderGraph::new();
                let source = graph.import_texture("Color grid", &input);
                let image = if use_split {
                    vertical_split_screen(source, source, 0.5, &mut graph)
                } else {
                    resize_nearest(source, ImageSize::Target, &mut graph)
                };
                let output = if input.format() == Format::Rgba32Float {
                    tone_map(image, Format::Rgba8UnormSrgb, &mut graph)
                } else {
                    image
                };
                graph.export(output);
                executor.execute(graph, &mut target).unwrap();
                let pixels = target.read_image(executor.wgpu_context()).await.unwrap();
                for (x, y, actual) in pixels.enumerate_pixels() {
                    assert_pixel(actual.0, baseline.get_pixel(x, y).0);
                }
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
