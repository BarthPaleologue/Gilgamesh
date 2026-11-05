#[test]
fn shaders_are_valid() {
    for source in [include_str!("evolve.wgsl"), include_str!("randomize.wgsl")] {
        let module = naga::front::wgsl::parse_str(source)
            .unwrap_or_else(|error| panic!("{}", error.emit_to_string(source)));
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::empty(),
        )
        .validate(&module)
        .unwrap_or_else(|error| panic!("{}", error.emit_to_string(source)));
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod gpu {
    use super::super::*;
    use gilgamesh::core::HeadlessRuntime;
    use std::collections::BTreeSet;
    use std::time::Duration;

    #[test]
    #[ignore = "GPU end-to-end check; run explicitly with approval"]
    #[expect(
        clippy::too_many_lines,
        reason = "Keeps GPU fixture setup, execution and assertions together"
    )]
    fn patterns_evolve_with_ping_pong_while_pause_and_custom_capture_preserve_state() {
        pollster::block_on(async {
            let mut runtime = HeadlessRuntime::<GameOfLife>::new(GRID_SIZE, GRID_SIZE).await;
            runtime.render(Duration::ZERO);
            let seed = runtime.capture_frame().await.unwrap();
            let population = live_cells(&seed).len();
            assert!(
                (GRID_SIZE * GRID_SIZE / 2..GRID_SIZE * GRID_SIZE * 3 / 4)
                    .contains(&(u32::try_from(population).unwrap()))
            );
            assert_eq!(runtime.app().seed, 1);
            let block = [(8, 8), (9, 8), (8, 9), (9, 9)];
            let blinker = [(20, 20), (21, 20), (22, 20)];
            let glider = [(41, 40), (42, 41), (40, 42), (41, 42), (42, 42)];
            let wrapped_blinker = [(127, 64), (0, 64), (1, 64)];
            let initial_cells: BTreeSet<_> = block
                .into_iter()
                .chain(blinker)
                .chain(glider)
                .chain(wrapped_blinker)
                .collect();
            let mut cells = vec![0; (GRID_SIZE * GRID_SIZE * 4) as usize];
            for pixel in cells.as_chunks_mut::<4>().0 {
                pixel[3] = 255;
            }
            for &(x, y) in &initial_cells {
                let index = ((y * GRID_SIZE + x) * 4) as usize;
                cells[index..index + 3].fill(255);
            }
            let queue = runtime.wgpu_context().queue.clone();
            queue.write_texture(
                runtime.app().states[0].as_image_copy(),
                &cells,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(GRID_SIZE * 4),
                    rows_per_image: Some(GRID_SIZE),
                },
                runtime.app().states[0].size(),
            );
            runtime.render(Duration::ZERO);
            let initial = runtime.capture_frame().await.unwrap();
            assert_eq!(live_cells(&initial), initial_cells);

            let half_step = Duration::from_secs_f64(STEP_SECONDS / 2.0);
            runtime.render(half_step);
            assert_eq!(runtime.app().current, 0);
            assert!(!runtime.app().step_due);
            assert_eq!(runtime.capture_frame().await.unwrap(), initial);
            runtime.render(half_step);
            assert_eq!(runtime.app().current, 1);
            assert!(runtime.app().step_due);
            let first_cells = block
                .into_iter()
                .chain([(21, 19), (21, 20), (21, 21)])
                .chain([(40, 41), (42, 41), (41, 42), (42, 42), (41, 43)])
                .chain([(0, 63), (0, 64), (0, 65)])
                .collect();
            let first = runtime.capture_frame().await.unwrap();
            assert_eq!(live_cells(&first), first_cells);

            runtime.render(Duration::from_millis(250));
            assert_eq!(runtime.app().current, 0);
            let second = runtime.capture_frame().await.unwrap();
            let remainder = runtime.app().accumulated_seconds;
            assert!(remainder > STEP_SECONDS);
            let custom = runtime.render_image(257, 193).await.unwrap();
            assert_eq!(custom.dimensions(), (257, 193));
            for (x, y, actual) in custom.enumerate_pixels() {
                let sx = (2 * x + 1) * GRID_SIZE / (2 * custom.width());
                let sy = (2 * y + 1) * GRID_SIZE / (2 * custom.height());
                assert_eq!(actual, second.get_pixel(sx, sy));
            }
            assert_eq!(runtime.app().current, 0);
            assert_eq!(runtime.app().accumulated_seconds, remainder);
            assert!(runtime.app().step_due);
            assert_eq!(runtime.app().seed, 1);
            assert_eq!(runtime.capture_frame().await.unwrap(), second);

            runtime.app_mut().paused = true;
            runtime.render(Duration::from_secs(10));
            assert_eq!(runtime.app().current, 0);
            assert_eq!(runtime.app().accumulated_seconds, remainder);
            assert_eq!(runtime.capture_frame().await.unwrap(), second);

            runtime.app_mut().paused = false;
            runtime.render(Duration::from_millis(1));
            assert_eq!(runtime.app().current, 1);
            runtime.render(half_step);
            assert_eq!(runtime.app().current, 0);
            assert!((runtime.app().accumulated_seconds - 0.001).abs() < 1e-9);
            let fourth_cells = block
                .into_iter()
                .chain(blinker)
                .chain(glider.map(|(x, y)| (x + 1, y + 1)))
                .chain(wrapped_blinker)
                .collect();
            let fourth = runtime.capture_frame().await.unwrap();
            assert_eq!(live_cells(&fourth), fourth_cells);

            runtime.resize(256, 256);
            runtime.render(Duration::ZERO);
            let resized = runtime.capture_frame().await.unwrap();
            for (x, y, actual) in resized.enumerate_pixels() {
                assert_eq!(actual, fourth.get_pixel(x / 2, y / 2));
            }
            assert_eq!(runtime.app().current, 0);
            assert_eq!(runtime.app().states[0].width(), GRID_SIZE);

            runtime.app_mut().reset_requested = true;
            runtime.resize(GRID_SIZE, GRID_SIZE);
            runtime.render(Duration::ZERO);
            assert_eq!(runtime.app().current, 0);
            assert_eq!(runtime.app().accumulated_seconds, 0.0);
            assert!(runtime.app().randomize_due);
            assert_eq!(runtime.app().seed, 2);
            let reset = runtime.capture_frame().await.unwrap();
            assert_ne!(reset, seed);
            let population = live_cells(&reset).len();
            assert!(
                (GRID_SIZE * GRID_SIZE / 2..GRID_SIZE * GRID_SIZE * 3 / 4)
                    .contains(&(u32::try_from(population).unwrap()))
            );
            assert_eq!(
                runtime.render_image(GRID_SIZE, GRID_SIZE).await.unwrap(),
                reset
            );
            assert!(!runtime.app().step_due);
            assert!(runtime.app().randomize_due);
            assert_eq!(runtime.app().seed, 2);

            runtime.app_mut().reset_requested = true;
            runtime.render(Duration::from_millis(100));
            assert!(runtime.app().randomize_due && runtime.app().step_due);
            assert_eq!(runtime.app().seed, 3);
            let combined = runtime.capture_frame().await.unwrap();
            runtime.app_mut().seed = 2;
            runtime.app_mut().reset_requested = true;
            runtime.render(Duration::ZERO);
            runtime.render(Duration::from_millis(100));
            assert_eq!(runtime.app().seed, 3);
            assert_eq!(runtime.capture_frame().await.unwrap(), combined);
        });
    }

    fn live_cells(image: &image::RgbaImage) -> BTreeSet<(u32, u32)> {
        image
            .enumerate_pixels()
            .filter_map(|(x, y, pixel)| {
                assert!(pixel[0] == 0 || pixel[0] == 255);
                assert_eq!(pixel.0, [pixel[0], pixel[0], pixel[0], 255]);
                (pixel[0] == 255).then_some((x, y))
            })
            .collect()
    }
}
