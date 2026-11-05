#![cfg(not(target_arch = "wasm32"))]

use gilgamesh::core::graph_executor::{FrameTarget, GraphExecutor};
use gilgamesh::core::render_graph::{
    Binding, ImageDescriptor, ImageSize, RenderGraph, TextureFormat, fragment_pass,
};
use gilgamesh_egui::egui::{self, Color32, Pos2, Rect, Vec2};
use gilgamesh_egui::{Egui, egui_overlay};

fn input(width: u32, height: u32, pixels_per_point: f32) -> egui::RawInput {
    #[expect(
        clippy::cast_precision_loss,
        reason = "Small test dimensions fit f32 exactly"
    )]
    let screen = Vec2::new(width as f32, height as f32) / pixels_per_point;
    let mut input = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, screen)),
        ..Default::default()
    };
    input
        .viewports
        .get_mut(&egui::ViewportId::ROOT)
        .expect("RawInput contains the root viewport")
        .native_pixels_per_point = Some(pixels_per_point);
    input
}

fn render(
    gui: &Egui,
    executor: &mut GraphExecutor,
    target: &mut FrameTarget,
    format: TextureFormat,
) {
    let mut graph = RenderGraph::new();
    let background = graph.create_image(
        "Background",
        ImageDescriptor {
            size: ImageSize::Target,
            format,
        },
    );
    fragment_pass(
        "Background",
        "@group(0) @binding(0) var<uniform> color: vec4<f32>;\n\
         @fragment fn fs_main() -> @location(0) vec4<f32> { return color; }",
        [Binding::uniform([0.0_f32, 0.0, 1.0, 1.0])],
        &[background],
        &mut graph,
    );
    let image = egui_overlay(gui, background, &mut graph);
    let image = if format == TextureFormat::Bgra8UnormSrgb {
        let rgba = graph.create_image(
            "RGBA capture",
            ImageDescriptor {
                size: ImageSize::Target,
                format: TextureFormat::Rgba8UnormSrgb,
            },
        );
        fragment_pass(
            "RGBA capture",
            "@group(0) @binding(0) var source: texture_2d<f32>;\n\
             @fragment fn fs_main(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> { return textureLoad(source, vec2<i32>(p.xy), 0); }",
            [Binding::Texture(image)],
            &[rgba],
            &mut graph,
        );
        rgba
    } else {
        image
    };
    graph.export(image);
    executor
        .execute(graph, target)
        .expect("egui graph must execute");
}

#[test]
#[ignore = "GPU end-to-end check; run explicitly with approval"]
fn ui_draws_over_existing_scene_and_tracks_size_and_scale() {
    pollster::block_on(async {
        let mut executor = GraphExecutor::new().await;
        let mut gui = Egui::default();
        let mut target = executor.create_target(80, 60);
        for (width, height, scale, covered, additive) in [
            (80, 60, 1.0, (10, 10), (35, 10)),
            (120, 100, 2.0, (20, 20), (70, 20)),
        ] {
            executor.resize_target(&mut target, width, height);
            gui.run_with_input(input(width, height, scale), |ctx| {
                let painter = ctx.layer_painter(egui::LayerId::background());
                painter.rect_filled(
                    Rect::from_min_max(Pos2::new(5.0, 5.0), Pos2::new(25.0, 25.0)),
                    0.0,
                    Color32::from_rgba_unmultiplied(255, 0, 0, 128),
                );
                painter.rect_filled(
                    Rect::from_min_max(Pos2::new(30.0, 5.0), Pos2::new(45.0, 25.0)),
                    0.0,
                    Color32::from_rgb_additive(255, 0, 0),
                );
            });
            render(
                &gui,
                &mut executor,
                &mut target,
                TextureFormat::Rgba8UnormSrgb,
            );
            let image = target.read_image(executor.wgpu_context()).await.unwrap();
            let pixel = image.get_pixel(covered.0, covered.1);
            for (actual, expected) in pixel.0.into_iter().zip([128, 0, 187, 255]) {
                assert!(actual.abs_diff(expected) <= 2, "{pixel:?}");
            }
            assert_eq!(image.get_pixel(width - 1, height - 1).0, [0, 0, 255, 255]);
            assert_eq!(
                image.get_pixel(additive.0, additive.1).0,
                [255, 0, 255, 255]
            );
            render(
                &gui,
                &mut executor,
                &mut target,
                TextureFormat::Rgba8UnormSrgb,
            );
            assert_eq!(
                target.read_image(executor.wgpu_context()).await.unwrap(),
                image
            );
        }
    });
}

#[test]
#[ignore = "GPU end-to-end check; run explicitly with approval"]
#[expect(
    clippy::too_many_lines,
    reason = "Exercises one texture lifecycle across updates, format changes and frame replay"
)]
fn texture_updates_fonts_and_retired_textures_survive_frame_replay() {
    pollster::block_on(async {
        let mut executor = GraphExecutor::new().await;
        let mut gui = Egui::default();
        let mut target = executor.create_target(100, 80);
        let mut texture = gui.context().load_texture(
            "Color",
            egui::ColorImage::filled([2, 2], Color32::RED),
            egui::TextureOptions::NEAREST,
        );
        for color in [Color32::RED, Color32::GREEN] {
            texture.set(
                egui::ColorImage::filled([2, 2], color),
                egui::TextureOptions::NEAREST,
            );
            gui.run_with_input(input(100, 80, 1.0), |ctx| {
                let painter = ctx.layer_painter(egui::LayerId::background());
                paint_texture(ctx, texture.id());
                painter.text(
                    Pos2::new(5.0, 35.0),
                    egui::Align2::LEFT_TOP,
                    "egui text",
                    egui::FontId::proportional(16.0),
                    Color32::WHITE,
                );
            });
            render(
                &gui,
                &mut executor,
                &mut target,
                TextureFormat::Rgba8UnormSrgb,
            );
            let image = target.read_image(executor.wgpu_context()).await.unwrap();
            assert_eq!(image.get_pixel(10, 10).0, color.to_array());
            for format in [TextureFormat::Bgra8UnormSrgb, TextureFormat::Rgba8UnormSrgb] {
                render(&gui, &mut executor, &mut target, format);
                assert_eq!(
                    target.read_image(executor.wgpu_context()).await.unwrap(),
                    image
                );
            }
            assert!(
                image
                    .enumerate_pixels()
                    .any(|(_, y, p)| y > 35 && p[0] > 0 && p[1] > 0)
            );
        }
        texture.set_partial(
            [1, 0],
            egui::ColorImage::filled([1, 1], Color32::RED),
            egui::TextureOptions::NEAREST,
        );
        gui.run_with_input(input(100, 80, 1.0), |ctx| {
            paint_texture(ctx, texture.id());
        });
        render(
            &gui,
            &mut executor,
            &mut target,
            TextureFormat::Rgba8UnormSrgb,
        );
        let partial = target.read_image(executor.wgpu_context()).await.unwrap();
        assert_eq!(partial.get_pixel(10, 10).0, Color32::GREEN.to_array());
        assert_eq!(partial.get_pixel(20, 10).0, Color32::RED.to_array());
        render(
            &gui,
            &mut executor,
            &mut target,
            TextureFormat::Bgra8UnormSrgb,
        );
        assert_eq!(
            target.read_image(executor.wgpu_context()).await.unwrap(),
            partial
        );
        // Drop a texture in the same UI pass that paints it; captures must still replay that frame.
        let mut texture = Some(texture);
        gui.run_with_input(input(100, 80, 1.0), |ctx| {
            if let Some(texture) = texture.take() {
                paint_texture(ctx, texture.id());
            }
        });
        render(
            &gui,
            &mut executor,
            &mut target,
            TextureFormat::Rgba8UnormSrgb,
        );
        let first = target.read_image(executor.wgpu_context()).await.unwrap();
        assert_eq!(first.get_pixel(10, 10).0, Color32::GREEN.to_array());
        render(
            &gui,
            &mut executor,
            &mut target,
            TextureFormat::Rgba8UnormSrgb,
        );
        assert_eq!(
            target.read_image(executor.wgpu_context()).await.unwrap(),
            first
        );
        gui.run_with_input(input(100, 80, 1.0), |_| {});
        render(
            &gui,
            &mut executor,
            &mut target,
            TextureFormat::Rgba8UnormSrgb,
        );
        let empty = target.read_image(executor.wgpu_context()).await.unwrap();
        assert!(empty.pixels().all(|p| p.0 == [0, 0, 255, 255]));
    });
}

fn paint_texture(context: &egui::Context, texture: egui::TextureId) {
    context.layer_painter(egui::LayerId::background()).image(
        texture,
        Rect::from_min_max(Pos2::new(5.0, 5.0), Pos2::new(25.0, 25.0)),
        Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
        Color32::WHITE,
    );
}
