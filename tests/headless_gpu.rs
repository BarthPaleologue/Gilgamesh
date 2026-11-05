#![cfg(not(target_arch = "wasm32"))]

use gilgamesh::blocks::{rasterize, tone_map};
use gilgamesh::camera::{Camera, Orthographic};
use gilgamesh::core::app::{App, FrameContext};
use gilgamesh::core::color_pass::ColorPass;
use gilgamesh::core::graph_executor::{FrameTarget, GraphExecutor};
use gilgamesh::core::headless_runtime::HeadlessRuntime;
use gilgamesh::core::render_context::FrameBindingsData;
use gilgamesh::core::render_graph::{RenderGraph, TextureFormat};
use gilgamesh::core::wgpu_context::WGPUContext;
use gilgamesh::core::window_runtime::WindowContext;
use gilgamesh::geometry::{mesh::Mesh, primitive};
use gilgamesh::input::mouse::Mouse;
use gilgamesh::lights::directional_light::DirectionalLight;
use gilgamesh::lights::light::Light;
use gilgamesh::materials::pbr::PbrMaterial;
use gilgamesh::transform::Transformable;
use glam::DVec3;
use std::cell::Cell;
use std::io::Cursor;
use std::rc::Rc;
use std::time::Duration;

#[test]
#[ignore = "GPU end-to-end check; run explicitly with approval"]
#[expect(
    clippy::too_many_lines,
    reason = "Keeps GPU fixture setup, execution and assertions together"
)]
fn graph_execution_and_captures_keep_targets_and_hooks_independent() {
    let first = pollster::block_on(async {
        let mut executor = GraphExecutor::new().await;
        let mut target = executor.create_target(67, 33);
        let mut app = TestApp::new(executor.wgpu_context());
        assert_eq!(
            target.output().texture().format(),
            wgpu::TextureFormat::Rgba8UnormSrgb
        );

        render_frame(&mut executor, &mut app, &mut target, 0.0, 0.0);
        let first = target.read_image(executor.wgpu_context()).await.unwrap();
        assert_eq!(first.dimensions(), (67, 33));
        assert_pixel(first.get_pixel(33, 16).0, [253, 182, 158, 255]);
        assert_square_footprint(&first);
        let projection = app.camera.projection;
        let background = first.get_pixel(0, 0).0;
        assert_ne!(background, first.get_pixel(33, 16).0);
        assert_eq!(first.get_pixel(66, 32).0, background);
        assert_eq!(blit_and_read(&executor, &target).await, first);
        assert_eq!(app.name(), "67x33 at 0.00");

        let first_view = target.output().clone();
        assert!(!executor.resize_target(&mut target, 67, 33));
        assert_eq!(first_view, *target.output());
        render_frame(&mut executor, &mut app, &mut target, 0.02, 0.02);
        assert_eq!(
            target.read_image(executor.wgpu_context()).await.unwrap(),
            first
        );

        let mut png = Cursor::new(Vec::new());
        first.write_to(&mut png, image::ImageFormat::Png).unwrap();
        assert_eq!(
            image::load_from_memory(png.get_ref()).unwrap().to_rgba8(),
            first
        );

        {
            let mut context = WindowContext {
                executor: &mut executor,
                frame: &target,
                elapsed_seconds: 0.02,
            };
            assert_eq!(context.capture_frame().await.unwrap(), first);
            assert_eq!(app.frames.len(), 2);
            let custom = context.render_image(&app, 65, 129).await.unwrap();
            assert_eq!(custom.dimensions(), (65, 129));
            assert_square_footprint(&custom);
            assert_pixel(custom.get_pixel(32, 64).0, [253, 182, 158, 255]);
            assert_eq!(context.capture_frame().await.unwrap(), first);
            assert_eq!(app.frames, [(0.0, 0.0, 67, 33), (0.02, 0.02, 67, 33)]);
            assert_eq!(app.name(), "67x33 at 0.02");
            assert_eq!(app.camera.projection, projection);
        }
        assert_eq!(app.frames.len(), 2);
        render_frame(&mut executor, &mut app, &mut target, 0.04, 0.02);
        assert_eq!(app.name(), "67x33 at 0.04");
        assert_eq!(app.camera.projection, projection);
        assert_pixel(
            target
                .read_image(executor.wgpu_context())
                .await
                .unwrap()
                .get_pixel(33, 16)
                .0,
            [150, 150, 253, 255],
        );

        assert!(executor.resize_target(&mut target, 3, 4));
        assert_ne!(first_view, *target.output());
        render_frame(&mut executor, &mut app, &mut target, 0.06, 0.02);
        let image = target.read_image(executor.wgpu_context()).await.unwrap();
        assert_eq!(image.dimensions(), (3, 4));
        assert_pixel(image.get_pixel(1, 2).0, [150, 150, 253, 255]);
        assert_eq!(blit_and_read(&executor, &target).await, image);
        assert_eq!(first_view.texture().width(), 67);
        assert_eq!(first_view.texture().height(), 33);
        assert_eq!(app.frames.len(), 4);

        first
    });

    let completed = Rc::new(Cell::new(false));
    let finished = completed.clone();
    HeadlessRuntime::<TestApp>::run(67, 33, |mut runtime| async move {
        runtime.app_mut().camera.projection.size_vertical = 5.0;
        runtime.render(Duration::from_millis(20));
        assert_eq!(runtime.app().frames, [(0.02, 0.02, 67, 33)]);
        let initial = runtime.capture_frame().await.unwrap();
        assert_ne!(initial, first);
        assert_pixel(initial.get_pixel(33, 16).0, [253, 182, 158, 255]);
        let pending = runtime.capture_frame();
        runtime.render(Duration::from_millis(20));
        assert_eq!(pending.await.unwrap(), initial);
        assert_eq!(runtime.app().frames.len(), 2);
        let main = runtime.capture_frame().await.unwrap();
        assert_pixel(main.get_pixel(33, 16).0, [150, 150, 253, 255]);
        let custom = runtime.render_image(129, 65);
        assert_eq!(
            runtime.app().frames,
            [(0.02, 0.02, 67, 33), (0.04, 0.02, 67, 33)]
        );
        runtime.render(Duration::ZERO);
        assert_eq!(custom.await.unwrap().dimensions(), (129, 65));
        assert_eq!(runtime.capture_frame().await.unwrap(), main);
        runtime.resize(3, 4);
        runtime.render(Duration::ZERO);
        assert_eq!(runtime.capture_frame().await.unwrap().dimensions(), (3, 4));
        assert_eq!(runtime.output().texture().size().width, 3);
        let gpu_image = gilgamesh::core::readback::read_rgba8(
            &runtime.wgpu_context().device,
            &runtime.wgpu_context().queue,
            runtime.output().texture(),
        )
        .await
        .unwrap();
        assert_eq!(gpu_image.dimensions(), (3, 4));
        finished.set(true);
    });
    assert!(completed.get());
}

fn render_frame(
    executor: &mut GraphExecutor,
    app: &mut TestApp,
    target: &mut FrameTarget,
    elapsed_seconds: f64,
    delta_seconds: f64,
) {
    let texture = target.output().texture();
    let mouse = Mouse::new();
    app.update_before_frame(&FrameContext {
        elapsed_seconds,
        delta_seconds,
        render_width: texture.width(),
        render_height: texture.height(),
        mouse: &mouse,
        wgpu_context: executor.wgpu_context(),
    });
    executor
        .execute(app.frame_graph(), target)
        .expect("Failed to render the headless test frame");
}

struct TestApp {
    name: String,
    camera: Camera<Orthographic>,
    light: DirectionalLight,
    mesh: Mesh,
    frames: Vec<(f64, f64, u32, u32)>,
}

impl App for TestApp {
    fn new(ctx: &WGPUContext) -> Self {
        let camera = Camera::new(Orthographic {
            size_vertical: 4.0,
            near: 0.1,
            far: 10.0,
        });
        camera.transform_mut().position = DVec3::new(0.0, 3.0, 0.0);
        camera.transform_mut().look_at(DVec3::ZERO, DVec3::NEG_Z);
        let mut light = DirectionalLight::default();
        light.set_intensity(0.0);
        let mut material = PbrMaterial::new("HDR");
        material.set_emissive_color(4.0, 1.0, 0.25);
        Self {
            name: String::new(),
            camera,
            light,
            mesh: Mesh::from_cpu_buffers(
                "Plane",
                &primitive::plane(2, 2.0),
                Rc::new(material.into()),
                ctx,
            ),
            frames: Vec::new(),
        }
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn update_before_frame(&mut self, frame: &FrameContext<'_>) {
        self.name = format!(
            "{}x{} at {:.2}",
            frame.render_width, frame.render_height, frame.elapsed_seconds
        );
        assert_eq!(frame.mouse.delta, [0.0; 2]);
        assert_eq!(frame.mouse.scroll_delta_lines, 0.0);
        if frame.elapsed_seconds >= 0.04 {
            let mut blue = PbrMaterial::new("Blue");
            blue.set_emissive_color(0.0, 0.0, 4.0);
            self.mesh.set_material(Rc::new(blue.into()));
        }
        self.frames.push((
            frame.elapsed_seconds,
            frame.delta_seconds,
            frame.render_width,
            frame.render_height,
        ));
    }

    fn frame_graph(&self) -> RenderGraph<'_> {
        let mut graph = RenderGraph::new();
        let scene = rasterize(
            FrameBindingsData::new(&self.camera, &self.light, &[]),
            [&self.mesh],
            TextureFormat::Rgba16Float,
            &mut graph,
        );
        let image = tone_map(scene.color, TextureFormat::Rgba8UnormSrgb, &mut graph);
        graph.export(image);
        graph
    }
}

async fn blit_and_read(executor: &GraphExecutor, target: &FrameTarget) -> image::RgbaImage {
    let ctx = executor.wgpu_context();
    let output = ctx.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Headless Output Consumer"),
        size: target.output().texture().size(),
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let blit = ColorPass::image_blit(&ctx.device, output.format());
    let source = blit.bind_source(&ctx.device, target.output());
    let mut encoder = ctx
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    blit.encode(
        &mut encoder,
        &source,
        &output.create_view(&wgpu::TextureViewDescriptor::default()),
    );
    ctx.queue.submit([encoder.finish()]);
    gilgamesh::core::readback::read_rgba8(&ctx.device, &ctx.queue, &output)
        .await
        .expect("Failed to read the blitted test frame")
}

fn assert_pixel(actual: [u8; 4], expected: [u8; 4]) {
    for (actual, expected) in actual.into_iter().zip(expected) {
        assert!(actual.abs_diff(expected) <= 1, "{actual} != {expected}");
    }
}

fn assert_square_footprint(image: &image::RgbaImage) {
    let background = image.get_pixel(0, 0);
    let horizontal = (0..image.width())
        .filter(|&x| image.get_pixel(x, image.height() / 2) != background)
        .count();
    let vertical = (0..image.height())
        .filter(|&y| image.get_pixel(image.width() / 2, y) != background)
        .count();
    assert!(horizontal > 0 && vertical > 0);
    assert!(
        horizontal.abs_diff(vertical) <= 1,
        "Square plane rendered as {horizontal}x{vertical}"
    );
}
