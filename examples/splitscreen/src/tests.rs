#[cfg(not(target_arch = "wasm32"))]
mod gpu {
    use super::super::*;
    use gilgamesh::core::graph_executor::{FrameTarget, GraphExecutor};
    use gilgamesh::input::mouse::Mouse;
    use glam::DVec4;
    use winit::dpi::PhysicalPosition;
    use winit::event::DeviceId;

    #[test]
    #[ignore = "GPU end-to-end check; run explicitly with approval"]
    #[expect(
        clippy::cast_precision_loss,
        reason = "Small fixture dimensions are exactly representable as f32"
    )]
    fn mouse_controls_color_depth_split_across_render_sizes() {
        pollster::block_on(async {
            let mut executor = GraphExecutor::new().await;
            let mut target = executor.create_target(321, 241);
            let mut app = SplitScreenDemo::new(executor.wgpu_context());
            assert_eq!(app.split, 0.5);
            for (width, height, split) in [(321, 241, 0.5), (173, 119, 0.25)] {
                executor.resize_target(&mut target, width, height);

                move_cursor(&mut app, &mut executor, &target, f64::from(width) + 10.0);
                assert_eq!(app.split, 1.0);
                render(&mut app, &mut executor, &mut target);
                let color = target.read_image(executor.wgpu_context()).await.unwrap();
                assert!(color.pixels().any(|p| p[0].abs_diff(p[2]) > 10));

                move_cursor(&mut app, &mut executor, &target, -10.0);
                assert_eq!(app.split, 0.0);
                render(&mut app, &mut executor, &mut target);
                let depth = target.read_image(executor.wgpu_context()).await.unwrap();
                assert!(
                    depth
                        .pixels()
                        .all(|p| p[0] == p[1] && p[1] == p[2] && p[3] == 255)
                );
                assert!(depth.pixels().any(|p| p[0] > 32));
                for y in [height * 7 / 10, height * 9 / 10] {
                    let x = width / 2;
                    let expected = ground_depth(&app.camera, width, height, x, y);
                    let actual = depth.get_pixel(x, y)[0];
                    assert!(
                        actual.abs_diff(expected) <= 1,
                        "{width}x{height}, ({x}, {y}): {actual} != {expected}"
                    );
                }

                move_cursor(&mut app, &mut executor, &target, f64::from(width) * split);
                render(&mut app, &mut executor, &mut target);
                let mixed = target.read_image(executor.wgpu_context()).await.unwrap();
                assert_eq!(mixed.dimensions(), (width, height));
                for (x, y, actual) in mixed.enumerate_pixels() {
                    let expected = if (x as f32 + 0.5) < app.split * width as f32 {
                        color.get_pixel(x, y)
                    } else {
                        depth.get_pixel(x, y)
                    };
                    assert_eq!(actual, expected, "{width}x{height}, ({x}, {y})");
                }
            }
        });
    }

    fn move_cursor(
        app: &mut SplitScreenDemo,
        executor: &mut GraphExecutor,
        frame: &FrameTarget,
        x: f64,
    ) {
        app.on_window_event(
            &WindowEvent::CursorMoved {
                device_id: DeviceId::dummy(),
                position: PhysicalPosition::new(x, 0.0),
            },
            &mut WindowContext {
                executor,
                frame,
                elapsed_seconds: 0.0,
            },
        );
    }

    fn render(app: &mut SplitScreenDemo, executor: &mut GraphExecutor, target: &mut FrameTarget) {
        let size = target.output().texture().size();
        app.update_before_render(&FrameContext {
            elapsed_seconds: 0.0,
            delta_seconds: 0.0,
            render_width: size.width,
            render_height: size.height,
            mouse: &Mouse::new(),
            wgpu_context: executor.wgpu_context(),
        });
        executor.execute(app.render_graph(), target).unwrap();
    }

    #[expect(
        clippy::cast_possible_truncation,
        reason = "Expected color channels are rounded to UNORM bytes"
    )]
    #[expect(
        clippy::cast_sign_loss,
        reason = "Expected color channels are nonnegative UNORM values"
    )]
    fn ground_depth(camera: &Camera<Perspective>, width: u32, height: u32, x: u32, y: u32) -> u8 {
        let clip = DVec4::new(
            2.0 * (f64::from(x) + 0.5) / f64::from(width) - 1.0,
            1.0 - 2.0 * (f64::from(y) + 0.5) / f64::from(height),
            1.0,
            1.0,
        );
        let world = camera.projection_view().inverse() * clip;
        let origin = camera.transform().world_position();
        let ray = world.truncate() / world.w - origin;
        let ground = origin + ray * (-origin.y / ray.y);
        assert!(ground.x.abs() < 7.0 && ground.z.abs() < 7.0);
        let projected = camera.projection_view() * ground.extend(1.0);
        let depth = projected.z / projected.w;
        let srgb = if depth <= 0.003_130_8 {
            12.92 * depth
        } else {
            1.055 * depth.powf(1.0 / 2.4) - 0.055
        };
        (srgb * 255.0).round() as u8
    }
}
