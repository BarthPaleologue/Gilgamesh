use gilgamesh::camera::{Camera, Perspective};
use gilgamesh::input::mouse::Mouse;
use gilgamesh::input::orbit_control::OrbitControl;
use gilgamesh::transform::Transformable;
use glam::DVec3;
use winit::dpi::PhysicalPosition;
use winit::event::{DeviceId, MouseScrollDelta, TouchPhase, WindowEvent};

fn radius_after_scroll(delta: MouseScrollDelta) -> f64 {
    let mut mouse = Mouse::new();
    mouse.listen_to_event(&WindowEvent::MouseWheel {
        device_id: DeviceId::dummy(),
        delta,
        phase: TouchPhase::Moved,
    });
    let mut camera: Camera = Camera::default();
    OrbitControl::default().update(&mut camera, &mouse, 1.0 / 60.0);
    camera.transform().position.length()
}

#[test]
fn trackpad_pixels_zoom_like_wheel_lines_in_both_directions() {
    for direction in [-1.0, 1.0] {
        let wheel = radius_after_scroll(MouseScrollDelta::LineDelta(0.0, direction as f32));
        let trackpad = radius_after_scroll(MouseScrollDelta::PixelDelta(PhysicalPosition::new(
            0.0,
            direction * 40.0,
        )));
        assert!((wheel - trackpad).abs() < 1e-12);
        if direction > 0.0 {
            assert!(trackpad < 10.0);
        } else {
            assert!(trackpad > 10.0);
        }
    }
    let horizontal = radius_after_scroll(MouseScrollDelta::PixelDelta(PhysicalPosition::new(
        40.0, 0.0,
    )));
    assert!((horizontal - 10.0).abs() < 1e-12);
}

#[test]
fn native_pinch_zooms_without_wheel_input() {
    for delta in [-0.1, 0.1] {
        let mut mouse = Mouse::new();
        mouse.listen_to_event(&WindowEvent::PinchGesture {
            device_id: DeviceId::dummy(),
            delta,
            phase: TouchPhase::Moved,
        });
        let mut camera: Camera = Camera::default();
        OrbitControl::default().update(&mut camera, &mouse, 1.0 / 60.0);
        let radius = camera.transform().position.length();
        if delta > 0.0 {
            assert!(radius < 10.0);
        } else {
            assert!(radius > 10.0);
        }
    }
}

#[test]
fn orbit_input_and_zoom_smoothing_are_independent_of_frame_partitioning() {
    let mut camera: Camera = Camera::default();
    let mut fast_camera: Camera = Camera::default();
    let mut control = OrbitControl::default();
    let mut fast_control = OrbitControl::default();
    let input = Mouse {
        left_button_pressed: true,
        delta: [20.0, -10.0],
        scroll_delta_pixels: 40.0,
        ..Mouse::default()
    };
    control.update(&mut camera, &input, 0.2);
    fast_control.update(&mut fast_camera, &input, 1.0 / 60.0);
    for _ in 1..12 {
        fast_control.update(&mut fast_camera, &Mouse::default(), 1.0 / 60.0);
    }
    let transform = camera.transform();
    let fast_transform = fast_camera.transform();
    assert!((transform.position - fast_transform.position).length() < 1e-12);
    assert!((transform.rotation.dot(fast_transform.rotation).abs() - 1.0).abs() < 1e-12);
}

#[test]
fn controls_preserve_camera_projection_and_scale() {
    let mut camera = Camera::new(Perspective {
        fov_vertical: std::f64::consts::FRAC_PI_3,
        aspect_ratio: 1.5,
        near: 0.5,
    });
    camera.transform_mut().scaling = DVec3::new(2.0, 3.0, 4.0);
    let projection = camera.projection_matrix();
    let input = Mouse {
        left_button_pressed: true,
        delta: [12.0, -8.0],
        scroll_delta_lines: 1.0,
        ..Mouse::default()
    };
    OrbitControl::default().update(&mut camera, &input, 1.0 / 60.0);
    assert_eq!(camera.projection_matrix(), projection);
    assert_eq!(camera.transform().scaling, DVec3::new(2.0, 3.0, 4.0));
    assert!(camera.transform().rotation.is_normalized());
    assert!(camera.transform().position.length() > 0.0);
}

#[test]
fn zoom_stops_at_the_minimum_radius() {
    let mut camera: Camera = Camera::default();
    let input = Mouse {
        scroll_delta_pixels: 40_000.0,
        ..Mouse::default()
    };
    OrbitControl::default().update(&mut camera, &input, 1000.0);
    assert!((camera.transform().position.length() - 1.0).abs() < 1e-12);
}
