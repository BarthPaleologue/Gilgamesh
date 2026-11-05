#![cfg(not(target_arch = "wasm32"))]

use gilgamesh::blocks::{bloom, pyramid_blur, resize_bilinear, tone_map};
use gilgamesh::core::graph_executor::GraphExecutor;
use gilgamesh::core::render_graph::{
    Binding, Image, ImageDescriptor, ImageSize, RenderGraph, fragment_pass,
};
use wgpu::util::{DeviceExt, TextureDataOrder};
use wgpu::{TextureFormat as Format, TextureUsages};

const CONSTANT: &str = r"
@group(0) @binding(0) var<uniform> color: vec4f;
@fragment fn fs_main() -> @location(0) vec4f { return color; }
";
const COPY: &str = r"
@group(0) @binding(0) var source: texture_2d<f32>;
@fragment fn fs_main(@builtin(position) p: vec4f) -> @location(0) vec4f {
    return textureLoad(source, vec2i(p.xy), 0);
}
";

#[test]
#[ignore = "GPU end-to-end check; run explicitly with approval"]
fn bilinear_interpolates_rgba_in_linear_space_and_clamps_edges() {
    pollster::block_on(async {
        let mut executor = GraphExecutor::new().await;
        let texels = [
            [0_u8, 64, 240, 32],
            [255, 128, 8, 224],
            [80, 255, 128, 96],
            [200, 0, 32, 160],
        ];
        for format in [Format::Rgba8Unorm, Format::Rgba8UnormSrgb] {
            let source = executor.wgpu_context().device.create_texture_with_data(
                &executor.wgpu_context().queue,
                &wgpu::TextureDescriptor {
                    label: Some("Bilinear grid"),
                    size: wgpu::Extent3d {
                        width: 2,
                        height: 2,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format,
                    usage: TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                },
                TextureDataOrder::LayerMajor,
                bytemuck::cast_slice(&texels),
            );
            let mut target = executor.create_target(5, 3);
            for (width, height) in [(5, 3), (1, 1), (2, 2), (7, 1)] {
                executor.resize_target(&mut target, width, height);
                let mut graph = RenderGraph::new();
                let input = graph.import_texture("Grid", &source);
                let output = resize_bilinear(input, ImageSize::Target, &mut graph);
                assert_eq!(graph.image_descriptor(output).format, format);
                graph.export(output);
                executor.execute(graph, &mut target).unwrap();
                let pixels = target.read_image(executor.wgpu_context()).await.unwrap();
                for (x, y, actual) in pixels.enumerate_pixels() {
                    let px = (x as f32 + 0.5) / width as f32 * 2.0 - 0.5;
                    let py = (y as f32 + 0.5) / height as f32 * 2.0 - 0.5;
                    let mut expected = [0_u8; 4];
                    for (c, value) in expected.iter_mut().enumerate() {
                        let sample = |ix: i32, iy: i32| {
                            let v = f32::from(
                                texels[(iy.clamp(0, 1) * 2 + ix.clamp(0, 1)) as usize][c],
                            ) / 255.0;
                            if format.is_srgb() && c < 3 {
                                srgb_decode(v)
                            } else {
                                v
                            }
                        };
                        let ix = px.floor() as i32;
                        let iy = py.floor() as i32;
                        let top = mix(sample(ix, iy), sample(ix + 1, iy), px - px.floor());
                        let bottom =
                            mix(sample(ix, iy + 1), sample(ix + 1, iy + 1), px - px.floor());
                        let v = mix(top, bottom, py - py.floor());
                        let encoded = if format.is_srgb() && c < 3 {
                            srgb_encode(v)
                        } else {
                            v
                        };
                        *value = (encoded * 255.0).round() as u8;
                    }
                    assert_pixel(actual.0, expected, 2);
                }
            }
        }
    });
}

#[test]
#[ignore = "GPU end-to-end check; run explicitly with approval"]
fn bilinear_keeps_hdr_and_blur_and_bloom_preserve_uniform_light() {
    pollster::block_on(async {
        let mut executor = GraphExecutor::new().await;
        let mut target = executor.create_target(1, 1);
        let mut graph = RenderGraph::new();
        let reference = constant([4.0, 2.0, 0.0, 0.5], ImageSize::Target, &mut graph);
        let output = tone_map(reference, Format::Rgba8UnormSrgb, &mut graph);
        graph.export(output);
        executor.execute(graph, &mut target).unwrap();
        let expected = target
            .read_image(executor.wgpu_context())
            .await
            .unwrap()
            .get_pixel(0, 0)
            .0;
        let mut graph = RenderGraph::new();
        let source = graph.create_image(
            "HDR pair",
            ImageDescriptor {
                size: ImageSize::Fixed {
                    width: 2,
                    height: 1,
                },
                format: Format::Rgba16Float,
            },
        );
        fragment_pass(
            "HDR pair",
            r"
            @fragment fn fs_main(@builtin(position) p: vec4f) -> @location(0) vec4f {
                return select(vec4f(8, 0, 0, 0.25), vec4f(0, 4, 0, 0.75), p.x > 1.0);
            }",
            [],
            &[source],
            &mut graph,
        );
        let resized = resize_bilinear(source, ImageSize::Target, &mut graph);
        let output = tone_map(resized, Format::Rgba8UnormSrgb, &mut graph);
        graph.export(output);
        executor.execute(graph, &mut target).unwrap();
        assert_pixel(
            target
                .read_image(executor.wgpu_context())
                .await
                .unwrap()
                .get_pixel(0, 0)
                .0,
            expected,
            1,
        );

        let color = [8.0, 2.0, 0.125, 0.375];
        for (width, height) in [(19, 13), (1, 1), (1, 7), (32, 24)] {
            executor.resize_target(&mut target, width, height);
            let mut graph = RenderGraph::new();
            let source = constant(color, ImageSize::Target, &mut graph);
            let output = tone_map(source, Format::Rgba8UnormSrgb, &mut graph);
            graph.export(output);
            executor.execute(graph, &mut target).unwrap();
            let expected = target
                .read_image(executor.wgpu_context())
                .await
                .unwrap()
                .get_pixel(0, 0)
                .0;
            for levels in [0, 1, 3, 6] {
                for size in [
                    ImageSize::Target,
                    ImageSize::Fixed { width, height },
                    ImageSize::Scaled {
                        numerator: 2,
                        denominator: 2,
                    },
                ] {
                    let mut graph = RenderGraph::new();
                    let source = constant(color, size, &mut graph);
                    let output = if levels >= 3 {
                        pyramid_blur(source, levels, &mut graph)
                    } else {
                        bloom(source, levels, 0.04, &mut graph)
                    };
                    assert_eq!(
                        graph.image_descriptor(output),
                        graph.image_descriptor(source)
                    );
                    let output = tone_map(output, Format::Rgba8UnormSrgb, &mut graph);
                    graph.export(output);
                    executor.execute(graph, &mut target).unwrap();
                    for pixel in target
                        .read_image(executor.wgpu_context())
                        .await
                        .unwrap()
                        .pixels()
                    {
                        assert_pixel(pixel.0, expected, 1);
                    }
                }
            }
        }
    });
}

#[test]
#[ignore = "GPU end-to-end check; run explicitly with approval"]
fn bloom_diffuses_subunit_light_and_uses_a_final_lerp_preserving_alpha() {
    pollster::block_on(async {
        let mut executor = GraphExecutor::new().await;
        let mut target = executor.create_target(19, 13);
        let mut captures = Vec::new();
        for weight in [None, Some(0.0), Some(1.0), Some(0.04)] {
            let mut graph = RenderGraph::new();
            let source = graph.create_image(
                "Dim patch",
                ImageDescriptor {
                    size: ImageSize::Target,
                    format: Format::Rgba16Float,
                },
            );
            fragment_pass(
                "Dim patch",
                r"
                @fragment fn fs_main(@builtin(position) p: vec4f) -> @location(0) vec4f {
                    let inside = all(p.xy >= vec2f(7, 4)) && all(p.xy < vec2f(12, 9));
                    return vec4f(vec3f(select(0.0, 0.5, inside)), select(0.25, 0.75, inside));
                }",
                [],
                &[source],
                &mut graph,
            );
            let filtered = match weight {
                Some(weight) => bloom(source, 4, weight, &mut graph),
                None => pyramid_blur(source, 4, &mut graph),
            };
            if weight == Some(0.0) {
                assert_eq!(filtered, source);
            }
            assert_eq!(pyramid_blur(source, 0, &mut graph), source);
            let output = graph.create_image(
                "Linear readback",
                ImageDescriptor {
                    size: ImageSize::Target,
                    format: Format::Rgba8Unorm,
                },
            );
            fragment_pass(
                "Linear readback",
                COPY,
                [Binding::Texture(filtered)],
                &[output],
                &mut graph,
            );
            graph.export(output);
            executor.execute(graph, &mut target).unwrap();
            captures.push(target.read_image(executor.wgpu_context()).await.unwrap());
        }
        let [blurred, original, diffuse, result] = captures.as_slice() else {
            unreachable!()
        };
        assert!(
            result
                .enumerate_pixels()
                .any(|(x, y, p)| original.get_pixel(x, y)[0] == 0 && p[0] > 0),
            "Subunit light must spread without a threshold"
        );
        assert!(
            result.get_pixel(9, 6)[0] < original.get_pixel(9, 6)[0],
            "Lerp must reduce the sharp peak"
        );
        assert!(
            blurred.enumerate_pixels().any(|(x, y, p)| {
                original.get_pixel(x, y)[0] == 0 && p[3] > original.get_pixel(x, y)[3]
            }),
            "Standalone blur must diffuse alpha as well as RGB"
        );
        for (a, b) in blurred.pixels().zip(diffuse.pixels()) {
            assert_eq!(
                a.0[..3],
                b.0[..3],
                "Bloom at weight 1 must use the standalone blur RGB"
            );
        }
        for (x, y, pixel) in result.enumerate_pixels() {
            let a = original.get_pixel(x, y);
            let b = diffuse.get_pixel(x, y);
            for c in 0..3 {
                let expected = mix(f32::from(a[c]), f32::from(b[c]), 0.04).round() as u8;
                assert!(pixel[c].abs_diff(expected) <= 1);
            }
            assert_eq!(pixel[3], a[3]);
        }
    });
}

fn constant(color: [f32; 4], size: ImageSize, graph: &mut RenderGraph<'_>) -> Image {
    let output = graph.create_image(
        "Constant HDR",
        ImageDescriptor {
            size,
            format: Format::Rgba16Float,
        },
    );
    fragment_pass(
        "Constant HDR",
        CONSTANT,
        [Binding::uniform(color)],
        &[output],
        graph,
    );
    output
}
fn assert_pixel(actual: [u8; 4], expected: [u8; 4], tolerance: u8) {
    for (a, b) in actual.into_iter().zip(expected) {
        assert!(a.abs_diff(b) <= tolerance, "{actual:?} != {expected:?}");
    }
}
fn mix(a: f32, b: f32, weight: f32) -> f32 {
    a * (1.0 - weight) + b * weight
}
fn srgb_decode(v: f32) -> f32 {
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}
fn srgb_encode(v: f32) -> f32 {
    if v <= 0.003_130_8 {
        v * 12.92
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    }
}
