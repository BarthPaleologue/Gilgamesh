#![cfg(not(target_arch = "wasm32"))]

use gilgamesh::blocks::{rasterize, tone_map};
use gilgamesh::camera::{Camera, Orthographic};
use gilgamesh::core::graph_executor::GraphExecutor;
use gilgamesh::core::render_context::FrameBindingsData;
use gilgamesh::core::render_graph::{Image, ImageDescriptor, ImageSize, PassContext, RenderGraph};
use gilgamesh::geometry::{mesh::Mesh, primitive};
use gilgamesh::lights::directional_light::DirectionalLight;
use gilgamesh::lights::light::Light;
use gilgamesh::materials::pbr::PbrMaterial;
use gilgamesh::transform::Transformable;
use glam::DVec3;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use wgpu::util::DeviceExt;
use wgpu::{TextureFormat as Format, TextureUsages as Usage};

#[test]
#[ignore = "GPU end-to-end check; run explicitly with approval"]
fn custom_blocks_mrt_compute_caches_and_independent_raster_inputs() {
    pollster::block_on(async {
        let mut executor = GraphExecutor::new().await;
        let mut target = executor.create_target(67, 33);
        let builds = Cell::new(0);
        let observed = RefCell::new(Vec::new());
        let mut previous = None;
        for split in [0.5, 0.25] {
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
            assert_pixel(
                pixels.get_pixel(20, 16).0,
                if split == 0.5 {
                    [150, 150, 253, 255]
                } else {
                    [253, 182, 158, 255]
                },
            );
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
                assert!(
                    !resources[..index].contains(texture),
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
        light.set_intensity(0.0);
        let mut material = PbrMaterial::new("Amber");
        material.set_ambient_color(4.0, 1.0, 0.25);
        let mesh = Mesh::from_cpu_buffers(
            "Plane",
            primitive::plane(2, 2.0),
            Rc::new(material.into()),
            gpu,
        );
        for format in [Format::Rgba16Float, Format::Rgba8Unorm] {
            let mut graph = RenderGraph::new();
            let left = rasterize(
                FrameBindingsData::new(&camera, &light, &[]),
                [&mesh],
                format,
                &mut graph,
            );
            let right = rasterize(
                FrameBindingsData::new(&other_camera, &light, &[]),
                [&mesh],
                format,
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

fn camera_at(x: f64) -> Camera<Orthographic> {
    let camera = Camera::new(Orthographic {
        size_vertical: 4.0,
        aspect_ratio: 67.0 / 33.0,
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
            const SOURCE: &str = r#"
            @vertex fn vs_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4f {
                let positions = array(vec2f(-1., -1.), vec2f(3., -1.), vec2f(-1., 3.));
                return vec4f(positions[index], 0., 1.);
            }
            struct Outputs { @location(0) amber: vec4f, @location(1) blue: vec4f }
            @fragment fn fs_main() -> Outputs {
                return Outputs(vec4f(4., 1., 0.25, 1.), vec4f(0., 0., 4., 1.));
            }
        "#;
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
            observed
                .borrow_mut()
                .extend([a, b].map(|image| context.images.get(image).texture().clone()));
            const SOURCE: &str = r#"
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
        "#;
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
                        compilation_options: Default::default(),
                        cache: None,
                    })
            });
            let layout = pipeline.get_bind_group_layout(0);
            let entries: Vec<_> = [inputs[0], inputs[1], a, b]
                .into_iter()
                .enumerate()
                .map(|(binding, image)| wgpu::BindGroupEntry {
                    binding: binding as u32,
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
            let mut pass = context.encoder.begin_compute_pass(&Default::default());
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
            observed.borrow_mut().extend([output].map(|image| context.images.get(image).texture().clone()));
        const SOURCE: &str = r#"
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
        "#;
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
            const SOURCE: &str = r#"
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
        "#;
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
                        compilation_options: Default::default(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: Some("fs_main"),
                        targets: &targets,
                        compilation_options: Default::default(),
                    }),
                    primitive: Default::default(),
                    depth_stencil: None,
                    multisample: Default::default(),
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
