#![cfg(not(target_arch = "wasm32"))]

use gilgamesh::blocks::{rasterize, tone_map};
use gilgamesh::camera::{Camera, Orthographic};
use gilgamesh::core::graph_executor::GraphExecutor;
use gilgamesh::core::render_context::FrameBindingsData;
use gilgamesh::core::render_graph::{Image, ImageDescriptor, ImageSize, PassContext, RenderGraph};
use gilgamesh::geometry::{mesh::Mesh, primitive};
use gilgamesh::lights::directional_light::DirectionalLight;
use gilgamesh::materials::pbr::PbrMaterial;
use gilgamesh::photometry::{Lux, Nits};
use gilgamesh::transform::Transformable;
use glam::DVec3;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use wgpu::util::DeviceExt;
use wgpu::{TextureFormat as Format, TextureUsages as Usage};

#[test]
#[ignore = "GPU end-to-end check; run explicitly with approval"]
fn raster_clear_color_preserves_rgba_between_frames() {
    pollster::block_on(async {
        let mut executor = GraphExecutor::new().await;
        let camera: Camera = Camera::default();
        let mut target = executor.create_target(8, 8);
        for clear_color in [
            [0.2, 0.4, 0.6, 0.25],
            [0.7, 0.1, 0.3, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ] {
            let mut graph = RenderGraph::new();
            let scene = rasterize(
                FrameBindingsData::new(&camera, &[], &[]),
                [],
                Format::Rgba8Unorm,
                clear_color,
                &mut graph,
            );
            graph.export(scene.color);
            executor.execute(graph, &mut target).unwrap();
            let image = target.read_image(executor.wgpu_context()).await.unwrap();
            for pixel in image.pixels() {
                for (actual, expected) in pixel.0.into_iter().zip(clear_color) {
                    assert!((f32::from(actual) - expected * 255.0).abs() <= 1.0);
                }
            }
        }
    });
}

#[test]
#[ignore = "GPU end-to-end check; run explicitly with approval"]
#[expect(
    clippy::too_many_lines,
    reason = "Keeps GPU fixture setup, execution and assertions together"
)]
fn custom_blocks_mrt_compute_caches_and_independent_raster_inputs() {
    pollster::block_on(async {
        let mut executor = GraphExecutor::new().await;
        let mut target = executor.create_target(67, 33);
        let builds = Cell::new(0);
        let observed = RefCell::new(Vec::new());
        let mut previous = None;
        for (split, middle_pixel) in [(0.5, [150, 150, 253, 255]), (0.25, [253, 182, 158, 255])] {
            let mut graph = RenderGraph::new();
            let [amber, blue] = mrt(&builds, &observed, &mut graph);
            let [blue, amber] = swap_images([amber, blue], &builds, &observed, &mut graph);
            let merged = vertical_split_screen(blue, amber, split, &builds, &observed, &mut graph);
            let output = tone_map(merged, Format::Rgba8UnormSrgb, &mut graph);
            graph.export(output);
            executor.execute(graph, &mut target).unwrap();
            let pixels = target.read_image(executor.wgpu_context()).await.unwrap();
            assert_pixel(pixels.get_pixel(8, 16).0, [150, 150, 253, 255]);
            assert_pixel(pixels.get_pixel(60, 16).0, [253, 182, 158, 255]);
            assert_pixel(pixels.get_pixel(20, 16).0, middle_pixel);
            if let Some(previous) = previous {
                assert_ne!(pixels, previous);
            }
            previous = Some(pixels);
        }
        assert_eq!(
            builds.get(),
            3,
            "Pipelines must survive graph reconstruction"
        );
        {
            let resources = observed.borrow();
            for (index, texture) in resources[..5].iter().enumerate() {
                let overlapping = if index < 4 {
                    &resources[..index]
                } else {
                    &resources[2..4]
                };
                assert!(
                    !overlapping.contains(texture),
                    "Simultaneous images must not alias"
                );
                assert!(
                    resources[5..].contains(texture),
                    "Intermediate textures must be reused across graphs"
                );
            }
        }

        let main = target.read_image(executor.wgpu_context()).await.unwrap();
        let mut custom = executor.create_target(19, 11);
        let mut graph = RenderGraph::new();
        let imported = graph.import_texture("Main frame", target.output().texture());
        assert_eq!(
            graph.import_texture("Same frame", target.output().texture()),
            imported
        );
        let output = resized_copy(imported, &builds, &mut graph);
        graph.export(output);
        executor.execute(graph, &mut custom).unwrap();
        let custom_pixels = custom.read_image(executor.wgpu_context()).await.unwrap();
        assert_eq!(custom_pixels.dimensions(), (19, 11));
        assert_pixel(custom_pixels.get_pixel(0, 5).0, [150, 150, 253, 255]);
        assert_pixel(custom_pixels.get_pixel(18, 5).0, [253, 182, 158, 255]);
        assert_eq!(
            target.read_image(executor.wgpu_context()).await.unwrap(),
            main
        );

        let output_before = custom.output().clone();
        let mut invalid = RenderGraph::new();
        let unwritten = image("Unwritten", Format::Rgba16Float, &mut invalid);
        invalid.export(unwritten);
        assert!(executor.execute(invalid, &mut custom).is_err());
        assert_eq!(output_before, *custom.output());
        assert_eq!(
            custom.read_image(executor.wgpu_context()).await.unwrap(),
            custom_pixels
        );

        let previous_image = target.output().texture().clone();
        let previous_view = target.output().clone();
        let mut graph = RenderGraph::new();
        let source = graph.import_texture("Previous frame", &previous_image);
        let output = resized_copy(source, &builds, &mut graph);
        graph.export(output);
        executor.execute(graph, &mut target).unwrap();
        assert_ne!(previous_view, *target.output());
        assert_eq!(
            target.read_image(executor.wgpu_context()).await.unwrap(),
            main
        );

        let mut graph = RenderGraph::new();
        let outputs = mrt(&builds, &observed, &mut graph);
        let output = tone_map(outputs[0], Format::Rgba8UnormSrgb, &mut graph);
        graph.export(output);
        executor.execute(graph, &mut target).unwrap();
        let pooled_import = observed.borrow()[observed.borrow().len() - 2].clone();
        let observed_before = observed.borrow().len();
        let mut graph = RenderGraph::new();
        let imported = graph.import_texture("Retained intermediate", &pooled_import);
        mrt(&builds, &observed, &mut graph);
        let output = tone_map(imported, Format::Rgba8UnormSrgb, &mut graph);
        graph.export(output);
        executor.execute(graph, &mut target).unwrap();
        assert!(
            !observed.borrow()[observed_before..].contains(&pooled_import),
            "A read-only import must be excluded from the intermediate pool"
        );
        assert_pixel(
            target
                .read_image(executor.wgpu_context())
                .await
                .unwrap()
                .get_pixel(33, 16)
                .0,
            [253, 182, 158, 255],
        );

        let gpu = executor.wgpu_context();
        let camera = camera_at(0.0);
        let other_camera = camera_at(10.0);
        let mut light = DirectionalLight::default();
        light.set_illuminance(Lux::new(0.0));
        let mut material = PbrMaterial::new("Amber");
        material.set_emissive_luminance(Nits::new(1.0));
        material.set_emissive_color(4.0, 1.0, 0.25);
        let mesh = Mesh::from_cpu_buffers(
            "Plane",
            &primitive::plane(2, 2.0),
            Rc::new(material.into()),
            gpu,
        );
        for format in [Format::Rgba16Float, Format::Rgba8Unorm] {
            let mut graph = RenderGraph::new();
            let left = rasterize(
                FrameBindingsData::new(&camera, std::slice::from_ref(&light), &[]),
                [&mesh],
                format,
                [0.1, 0.1, 0.2, 1.0],
                &mut graph,
            );
            let right = rasterize(
                FrameBindingsData::new(&other_camera, std::slice::from_ref(&light), &[]),
                [&mesh],
                format,
                [0.1, 0.1, 0.2, 1.0],
                &mut graph,
            );
            let output =
                vertical_split_screen(left.color, right.color, 0.5, &builds, &observed, &mut graph);
            let output = tone_map(output, Format::Rgba8UnormSrgb, &mut graph);
            graph.export(output);
            executor.execute(graph, &mut target).unwrap();
            let pixels = target.read_image(executor.wgpu_context()).await.unwrap();
            assert_ne!(
                pixels.get_pixel(28, 16),
                pixels.get_pixel(38, 16),
                "Raster passes must use their own cameras"
            );
            assert_eq!(pixels.get_pixel(38, 16), pixels.get_pixel(0, 0));
            if format == Format::Rgba16Float {
                assert_pixel(pixels.get_pixel(28, 16).0, [253, 182, 158, 255]);
            }
        }
    });
}

#[test]
#[ignore = "GPU end-to-end check; run explicitly with approval"]
#[expect(
    clippy::too_many_lines,
    reason = "Keeps GPU fixture setup, execution and assertions together"
)]
fn imported_textures_keep_written_state_across_ping_pong_frames() {
    pollster::block_on(async {
        let mut executor = GraphExecutor::new().await;
        let mut target = executor.create_target(8, 4);
        let gpu = executor.wgpu_context();
        let states = ["State A", "State B"].map(|label| {
            gpu.device.create_texture_with_data(
                &gpu.queue,
                &wgpu::TextureDescriptor {
                    label: Some(label),
                    size: wgpu::Extent3d {
                        width: 8,
                        height: 4,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: Format::Rgba8Unorm,
                    usage: Usage::TEXTURE_BINDING | Usage::STORAGE_BINDING,
                    view_formats: &[],
                },
                wgpu::util::TextureDataOrder::LayerMajor,
                &[0, 0, 0, 255].repeat(32),
            )
        });
        let shader = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Increment State"),
                source: wgpu::ShaderSource::Wgsl(
                    r"
                @group(0) @binding(0) var previous: texture_2d<f32>;
                @group(0) @binding(1) var next: texture_storage_2d<rgba8unorm, write>;
                @compute @workgroup_size(8, 8)
                fn main(@builtin(global_invocation_id) id: vec3u) {
                    if any(id.xy >= textureDimensions(next)) { return; }
                    let p = vec2i(id.xy);
                    textureStore(next, p, textureLoad(previous, p, 0) + vec4f(0.25, 0., 0., 0.));
                }
                "
                    .into(),
                ),
            });
        let pipeline = gpu
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Increment State"),
                layout: None,
                module: &shader,
                entry_point: Some("main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            });
        let builds = Cell::new(0);
        let mut front = 0;
        for (width, expected_red) in [(8, 137), (16, 188), (8, 225), (16, 255)] {
            executor.resize_target(&mut target, width, 4);
            let alias = states[1 - front].clone();
            let mut graph = RenderGraph::new();
            let previous = graph.import_texture("Previous", &states[front]);
            let next_texture = &states[1 - front];
            let next = graph.import_texture("Next", next_texture);
            assert_eq!(next, graph.import_texture("Same Next", &alias));
            let output = resized_copy(next, &builds, &mut graph);
            let pipeline = &pipeline;
            graph.add_pass(
                "Increment",
                &[(previous, Usage::TEXTURE_BINDING)],
                &[(next, Usage::STORAGE_BINDING)],
                move |context| {
                    assert_eq!(context.images.get(next).texture(), next_texture);
                    let bindings =
                        context
                            .gpu
                            .device
                            .create_bind_group(&wgpu::BindGroupDescriptor {
                                label: None,
                                layout: &pipeline.get_bind_group_layout(0),
                                entries: &[
                                    wgpu::BindGroupEntry {
                                        binding: 0,
                                        resource: wgpu::BindingResource::TextureView(
                                            context.images.get(previous),
                                        ),
                                    },
                                    wgpu::BindGroupEntry {
                                        binding: 1,
                                        resource: wgpu::BindingResource::TextureView(
                                            context.images.get(next),
                                        ),
                                    },
                                ],
                            });
                    let mut pass = context
                        .encoder
                        .begin_compute_pass(&wgpu::ComputePassDescriptor::default());
                    pass.set_pipeline(pipeline);
                    pass.set_bind_group(0, &bindings, &[]);
                    pass.dispatch_workgroups(1, 1, 1);
                },
            );
            graph.export(output);
            executor.execute(graph, &mut target).unwrap();
            front = 1 - front;
            for pixel in target
                .read_image(executor.wgpu_context())
                .await
                .unwrap()
                .pixels()
            {
                assert_pixel(pixel.0, [expected_red, 0, 0, 255]);
            }
        }

        let imported_graph = || {
            let mut graph = RenderGraph::new();
            let state = graph.import_texture("State", &states[front]);
            let output = resized_copy(state, &builds, &mut graph);
            graph.export(output);
            (graph, state)
        };
        let (mut graph, state) = imported_graph();
        graph.add_pass(
            "Unsupported Usage",
            &[],
            &[(state, Usage::RENDER_ATTACHMENT)],
            |_| {},
        );
        assert!(
            graph
                .validate()
                .unwrap_err()
                .to_string()
                .contains("lacks required usages")
        );

        let (mut graph, state) = imported_graph();
        graph.add_pass(
            "First Writer",
            &[],
            &[(state, Usage::STORAGE_BINDING)],
            |_| {},
        );
        graph.add_pass(
            "Second Writer",
            &[],
            &[(state, Usage::STORAGE_BINDING)],
            |_| {},
        );
        assert!(
            graph
                .validate()
                .unwrap_err()
                .to_string()
                .contains("multiple writers")
        );

        let texture_alias = states[front].clone();
        let (mut graph, state) = imported_graph();
        let alias = graph.import_texture("Same State", &texture_alias);
        assert_eq!(state, alias);
        graph.add_pass(
            "Feedback",
            &[(state, Usage::TEXTURE_BINDING)],
            &[(alias, Usage::STORAGE_BINDING)],
            |_| {},
        );
        assert!(
            graph
                .validate()
                .unwrap_err()
                .to_string()
                .contains("same physical texture")
        );
    });
}

fn camera_at(x: f64) -> Camera<Orthographic> {
    let camera = Camera::new(Orthographic {
        size_vertical: 4.0,
        near: 0.1,
        far: 10.0,
    });
    camera.transform_mut().position = DVec3::new(x, 3.0, 0.0);
    camera
        .transform_mut()
        .look_at(DVec3::new(x, 0.0, 0.0), DVec3::NEG_Z);
    camera
}

fn image(label: &str, format: Format, graph: &mut RenderGraph<'_>) -> Image {
    graph.create_image(
        label,
        ImageDescriptor {
            size: ImageSize::Target,
            format,
        },
    )
}

fn mrt<'a>(
    builds: &'a Cell<usize>,
    observed: &'a RefCell<Vec<wgpu::Texture>>,
    graph: &mut RenderGraph<'a>,
) -> [Image; 2] {
    let a = image("MRT Amber", Format::Rgba16Float, graph);
    let b = image("MRT Blue", Format::Rgba16Float, graph);
    graph.add_pass(
        "Custom MRT",
        &[],
        &[(a, Usage::RENDER_ATTACHMENT), (b, Usage::RENDER_ATTACHMENT)],
        move |context| {
            const SOURCE: &str = r"
            @vertex fn vs_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4f {
                let positions = array(vec2f(-1., -1.), vec2f(3., -1.), vec2f(-1., 3.));
                return vec4f(positions[index], 0., 1.);
            }
            struct Outputs { @location(0) amber: vec4f, @location(1) blue: vec4f }
            @fragment fn fs_main() -> Outputs {
                return Outputs(vec4f(4., 1., 0.25, 1.), vec4f(0., 0., 4., 1.));
            }
        ";
            observed.borrow_mut().extend([
                context.images.get(a).texture().clone(),
                context.images.get(b).texture().clone(),
            ]);
            let pipeline = render_pipeline(
                context,
                SOURCE,
                &[Format::Rgba16Float, Format::Rgba16Float],
                builds,
            );
            let attachments = [a, b].map(|image| {
                Some(wgpu::RenderPassColorAttachment {
                    view: context.images.get(image),
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })
            });
            let mut pass = context
                .encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    color_attachments: &attachments,
                    ..Default::default()
                });
            pass.set_pipeline(&pipeline);
            pass.draw(0..3, 0..1);
        },
    );
    [a, b]
}

fn swap_images<'a>(
    inputs: [Image; 2],
    builds: &'a Cell<usize>,
    observed: &'a RefCell<Vec<wgpu::Texture>>,
    graph: &mut RenderGraph<'a>,
) -> [Image; 2] {
    let a = image("Compute Blue", Format::Rgba16Float, graph);
    let b = image("Compute Amber", Format::Rgba16Float, graph);
    graph.add_pass(
        "Custom Compute",
        &inputs.map(|image| (image, Usage::TEXTURE_BINDING)),
        &[(a, Usage::STORAGE_BINDING), (b, Usage::STORAGE_BINDING)],
        move |context| {
            const SOURCE: &str = r"
            @group(0) @binding(0) var first: texture_2d<f32>;
            @group(0) @binding(1) var second: texture_2d<f32>;
            @group(0) @binding(2) var left: texture_storage_2d<rgba16float, write>;
            @group(0) @binding(3) var right: texture_storage_2d<rgba16float, write>;
            @compute @workgroup_size(8, 8) fn main(@builtin(global_invocation_id) id: vec3u) {
                if any(id.xy >= textureDimensions(left)) { return; }
                let p = vec2i(id.xy);
                textureStore(left, p, textureLoad(second, p, 0));
                textureStore(right, p, textureLoad(first, p, 0));
            }
        ";
            observed
                .borrow_mut()
                .extend([a, b].map(|image| context.images.get(image).texture().clone()));

            let pipeline = context.cache.get_or_insert_with(SOURCE, || {
                builds.set(builds.get() + 1);
                let shader =
                    context
                        .gpu
                        .device
                        .create_shader_module(wgpu::ShaderModuleDescriptor {
                            label: None,
                            source: wgpu::ShaderSource::Wgsl(SOURCE.into()),
                        });
                context
                    .gpu
                    .device
                    .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                        label: None,
                        layout: None,
                        module: &shader,
                        entry_point: Some("main"),
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                        cache: None,
                    })
            });
            let layout = pipeline.get_bind_group_layout(0);
            let entries: Vec<_> = [inputs[0], inputs[1], a, b]
                .into_iter()
                .enumerate()
                .map(|(binding, image)| wgpu::BindGroupEntry {
                    binding: u32::try_from(binding).expect("Shader binding index fits in u32"),
                    resource: wgpu::BindingResource::TextureView(context.images.get(image)),
                })
                .collect();
            let bindings = context
                .gpu
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: None,
                    layout: &layout,
                    entries: &entries,
                });
            let texture = context.images.get(a).texture();
            let mut pass = context
                .encoder
                .begin_compute_pass(&wgpu::ComputePassDescriptor::default());
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, &bindings, &[]);
            pass.dispatch_workgroups(texture.width().div_ceil(8), texture.height().div_ceil(8), 1);
        },
    );
    [a, b]
}

fn vertical_split_screen<'a>(
    left: Image,
    right: Image,
    split: f32,
    builds: &'a Cell<usize>,
    observed: &'a RefCell<Vec<wgpu::Texture>>,
    graph: &mut RenderGraph<'a>,
) -> Image {
    let output = image("Split", Format::Rgba16Float, graph);
    graph.add_pass("Custom Split", &[(left, Usage::TEXTURE_BINDING), (right, Usage::TEXTURE_BINDING)], &[(output, Usage::RENDER_ATTACHMENT)], move |context| {
        const SOURCE: &str = r"
            @group(0) @binding(0) var left: texture_2d<f32>;
            @group(0) @binding(1) var right: texture_2d<f32>;
            @group(0) @binding(2) var<uniform> split: f32;
            @vertex fn vs_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4f {
                let positions = array(vec2f(-1., -1.), vec2f(3., -1.), vec2f(-1., 3.));
                return vec4f(positions[index], 0., 1.);
            }
            @fragment fn fs_main(@builtin(position) p: vec4f) -> @location(0) vec4f {
                if p.x < split * f32(textureDimensions(left).x) { return textureLoad(left, vec2i(p.xy), 0); }
                return textureLoad(right, vec2i(p.xy), 0);
            }
        ";
            observed.borrow_mut().extend([output].map(|image| context.images.get(image).texture().clone()));

        let pipeline = render_pipeline(context, SOURCE, &[Format::Rgba16Float], builds);
        let uniform = context.gpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: None, contents: bytemuck::bytes_of(&split), usage: wgpu::BufferUsages::UNIFORM });
        let bindings = context.gpu.device.create_bind_group(&wgpu::BindGroupDescriptor { label: None, layout: &pipeline.get_bind_group_layout(0), entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(context.images.get(left)) },
            wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(context.images.get(right)) },
            wgpu::BindGroupEntry { binding: 2, resource: uniform.as_entire_binding() },
        ] });
        fullscreen(context, output, &pipeline, &bindings);
    });
    output
}

fn resized_copy<'a>(source: Image, builds: &'a Cell<usize>, graph: &mut RenderGraph<'a>) -> Image {
    let output = image("Resized Import", Format::Rgba8UnormSrgb, graph);
    graph.add_pass(
        "Resample",
        &[(source, Usage::TEXTURE_BINDING)],
        &[(output, Usage::RENDER_ATTACHMENT)],
        move |context| {
            const SOURCE: &str = r"
            @group(0) @binding(0) var source: texture_2d<f32>;
            struct Vertex { @builtin(position) p: vec4f, @location(0) uv: vec2f }
            @vertex fn vs_main(@builtin(vertex_index) index: u32) -> Vertex {
                let positions = array(vec2f(-1., -1.), vec2f(3., -1.), vec2f(-1., 3.));
                let p = positions[index];
                return Vertex(vec4f(p, 0., 1.), vec2f(p.x * 0.5 + 0.5, 0.5 - p.y * 0.5));
            }
            @fragment fn fs_main(v: Vertex) -> @location(0) vec4f {
                return textureLoad(source, vec2i(v.uv * vec2f(textureDimensions(source))), 0);
            }
        ";
            let pipeline = render_pipeline(context, SOURCE, &[Format::Rgba8UnormSrgb], builds);
            let bindings = context
                .gpu
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: None,
                    layout: &pipeline.get_bind_group_layout(0),
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(context.images.get(source)),
                    }],
                });
            fullscreen(context, output, &pipeline, &bindings);
        },
    );
    output
}

fn render_pipeline(
    context: &mut PassContext<'_>,
    source: &'static str,
    formats: &[Format],
    builds: &Cell<usize>,
) -> wgpu::RenderPipeline {
    context
        .cache
        .get_or_insert_with((source, formats.to_vec()), || {
            builds.set(builds.get() + 1);
            let shader = context
                .gpu
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: None,
                    source: wgpu::ShaderSource::Wgsl(source.into()),
                });
            let targets: Vec<_> = formats
                .iter()
                .map(|&format| {
                    Some(wgpu::ColorTargetState {
                        format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })
                })
                .collect();
            context
                .gpu
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: None,
                    layout: None,
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: Some("vs_main"),
                        buffers: &[],
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: Some("fs_main"),
                        targets: &targets,
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    }),
                    primitive: wgpu::PrimitiveState::default(),
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState::default(),
                    multiview_mask: None,
                    cache: None,
                })
        })
        .clone()
}

fn fullscreen(
    context: &mut PassContext<'_>,
    output: Image,
    pipeline: &wgpu::RenderPipeline,
    bindings: &wgpu::BindGroup,
) {
    let mut pass = context
        .encoder
        .begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: context.images.get(output),
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });
    pass.set_pipeline(pipeline);
    pass.set_bind_group(0, bindings, &[]);
    pass.draw(0..3, 0..1);
}

fn assert_pixel(actual: [u8; 4], expected: [u8; 4]) {
    for (actual, expected) in actual.into_iter().zip(expected) {
        assert!(actual.abs_diff(expected) <= 1, "{actual} != {expected}");
    }
}

#[test]
#[ignore = "GPU end-to-end check; run explicitly with approval"]
#[expect(
    clippy::too_many_lines,
    reason = "Keeps GPU fixture setup, execution and assertions together"
)]
fn versions_preserve_old_reads_before_ping_pong_overwrites() {
    pollster::block_on(async {
        let mut executor = GraphExecutor::new().await;
        let mut target = executor.create_target(8, 4);
        let saved = RefCell::new(Vec::new());
        let mut graph = RenderGraph::new();
        let a = image("A", Format::Rgba8Unorm, &mut graph);
        let b = image("B", Format::Rgba8Unorm, &mut graph);
        let next_a = graph.next_version(a);
        let next_b = graph.next_version(b);
        let snapshot = image("Old A", Format::Rgba8Unorm, &mut graph);
        for (source, destination) in [(next_b, next_a), (a, snapshot), (a, b)] {
            let saved = &saved;
            graph.add_pass(
                "Copy",
                &[(source, Usage::COPY_SRC)],
                &[(destination, Usage::COPY_DST)],
                move |context| {
                    if destination == next_a {
                        saved
                            .borrow_mut()
                            .push(context.images.get(destination).texture().clone());
                    }
                    if destination == snapshot {
                        saved
                            .borrow_mut()
                            .push(context.images.get(source).texture().clone());
                    }
                    context.encoder.copy_texture_to_texture(
                        context.images.get(source).texture().as_image_copy(),
                        context.images.get(destination).texture().as_image_copy(),
                        wgpu::Extent3d {
                            width: 8,
                            height: 4,
                            depth_or_array_layers: 1,
                        },
                    );
                },
            );
        }
        for (image, color) in [(a, wgpu::Color::RED), (next_b, wgpu::Color::GREEN)] {
            graph.add_pass(
                "Initialize",
                &[],
                &[(image, Usage::RENDER_ATTACHMENT)],
                move |context| {
                    let _pass = context
                        .encoder
                        .begin_render_pass(&wgpu::RenderPassDescriptor {
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: context.images.get(image),
                                depth_slice: None,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(color),
                                    store: wgpu::StoreOp::Store,
                                },
                            })],
                            ..Default::default()
                        });
                },
            );
        }
        let comparison =
            gilgamesh::blocks::vertical_split_screen(snapshot, next_a, 0.5, &mut graph);
        graph.export(comparison);
        executor.execute(graph, &mut target).unwrap();
        assert_eq!(
            saved.borrow()[0],
            saved.borrow()[1],
            "Logical versions must share their physical texture"
        );
        let pixels = target.read_image(executor.wgpu_context()).await.unwrap();
        for (x, _, pixel) in pixels.enumerate_pixels() {
            assert_eq!(
                pixel.0,
                if x < 4 {
                    [255, 0, 0, 255]
                } else {
                    [0, 255, 0, 255]
                }
            );
        }
        let mut graph = RenderGraph::new();
        let initial = image("Versioned output", Format::Rgba8Unorm, &mut graph);
        let final_version = graph.next_version(initial);
        for (image, color) in [
            (final_version, wgpu::Color::GREEN),
            (initial, wgpu::Color::RED),
        ] {
            graph.add_pass(
                "Write output version",
                &[],
                &[(image, Usage::RENDER_ATTACHMENT)],
                move |context| {
                    let _pass = context
                        .encoder
                        .begin_render_pass(&wgpu::RenderPassDescriptor {
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: context.images.get(image),
                                depth_slice: None,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(color),
                                    store: wgpu::StoreOp::Store,
                                },
                            })],
                            ..Default::default()
                        });
                },
            );
        }
        graph.export(final_version);
        executor.execute(graph, &mut target).unwrap();
        assert!(
            target
                .read_image(executor.wgpu_context())
                .await
                .unwrap()
                .pixels()
                .all(|pixel| pixel.0 == [0, 255, 0, 255])
        );
    });
}

#[test]
#[ignore = "GPU end-to-end check; run explicitly with approval"]
fn transient_chains_reuse_two_textures_without_overwriting_branches_or_other_formats() {
    pollster::block_on(async {
        let mut executor = GraphExecutor::new().await;
        let mut target = executor.create_target(8, 4);
        for (branch, different_format, expected_textures) in
            [(false, false, 3), (true, false, 4), (false, true, 4)]
        {
            let observed = RefCell::new(Vec::new());
            let mut graph = RenderGraph::new();
            let first = image("Initial", Format::Rgba8Unorm, &mut graph);
            graph.add_pass(
                "Initialize",
                &[],
                &[(first, Usage::RENDER_ATTACHMENT)],
                move |context| {
                    let _pass = context
                        .encoder
                        .begin_render_pass(&wgpu::RenderPassDescriptor {
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: context.images.get(first),
                                depth_slice: None,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(wgpu::Color {
                                        r: 0.25,
                                        g: 0.0,
                                        b: 0.0,
                                        a: 1.0,
                                    }),
                                    store: wgpu::StoreOp::Store,
                                },
                            })],
                            ..Default::default()
                        });
                },
            );
            observe_texture(first, &observed, &mut graph);
            let mut current = first;
            for index in 1..=8 {
                let format = if different_format && index == 3 {
                    Format::Rgba16Float
                } else {
                    Format::Rgba8Unorm
                };
                current = if graph.image_descriptor(current).format == format {
                    gilgamesh::blocks::resize_nearest(current, ImageSize::Target, &mut graph)
                } else {
                    tone_map(current, format, &mut graph)
                };
                observe_texture(current, &observed, &mut graph);
            }
            let output = if branch {
                let output =
                    gilgamesh::blocks::vertical_split_screen(first, current, 0.5, &mut graph);
                observe_texture(output, &observed, &mut graph);
                output
            } else {
                current
            };
            graph.export(output);
            executor.execute(graph, &mut target).unwrap();
            let pixels = target.read_image(executor.wgpu_context()).await.unwrap();
            assert!(pixels.pixels().all(|pixel| pixel.0 == [64, 0, 0, 255]));
            let observed = observed.borrow();
            let mut unique = Vec::new();
            for texture in observed.iter() {
                if !unique.contains(texture) {
                    unique.push(texture.clone());
                }
            }
            assert_eq!(unique.len(), expected_textures);
            for pair in observed.windows(2) {
                assert_ne!(pair[0], pair[1], "Input and output must stay distinct");
            }
            if branch {
                for texture in &observed[1..] {
                    assert_ne!(
                        observed[0], *texture,
                        "The first image stays live until the merge"
                    );
                }
            }
        }
    });
}

fn observe_texture<'a>(
    image: Image,
    observed: &'a RefCell<Vec<wgpu::Texture>>,
    graph: &mut RenderGraph<'a>,
) {
    graph.add_pass(
        "Observe allocation",
        &[(image, Usage::TEXTURE_BINDING)],
        &[],
        move |context| {
            observed
                .borrow_mut()
                .push(context.images.get(image).texture().clone());
        },
    );
}
