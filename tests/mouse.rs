use gilgamesh::input::mouse::Mouse;
use winit::dpi::PhysicalPosition;
use winit::event::{DeviceId, MouseScrollDelta, TouchPhase, WindowEvent};

#[test]
fn scroll_input_accumulates_until_the_frame_is_consumed() {
    let mut mouse = Mouse::new();
    for delta in [1.0, 2.0] {
        mouse.listen_to_event(&WindowEvent::MouseWheel {
            device_id: DeviceId::dummy(),
            delta: MouseScrollDelta::LineDelta(0.0, delta),
            phase: TouchPhase::Moved,
        });
    }
    mouse.listen_to_event(&WindowEvent::RedrawRequested);
    assert_eq!(mouse.scroll_delta_lines, 3.0);

    mouse.reset();
    assert_eq!(mouse.scroll_delta_lines, 0.0);
}

#[test]
fn wheel_trackpad_and_pinch_accumulate_in_their_own_units_until_reset() {
    let mut mouse = Mouse::new();
    for delta in [
        MouseScrollDelta::LineDelta(0.0, 1.0),
        MouseScrollDelta::PixelDelta(PhysicalPosition::new(7.0, 12.5)),
        MouseScrollDelta::PixelDelta(PhysicalPosition::new(-3.0, 7.5)),
    ] {
        mouse.listen_to_event(&WindowEvent::MouseWheel {
            device_id: DeviceId::dummy(),
            delta,
            phase: TouchPhase::Moved,
        });
    }
    for delta in [0.25, -0.125] {
        mouse.listen_to_event(&WindowEvent::PinchGesture {
            device_id: DeviceId::dummy(),
            delta,
            phase: TouchPhase::Moved,
        });
    }
    mouse.listen_to_event(&WindowEvent::MouseWheel {
        device_id: DeviceId::dummy(),
        delta: MouseScrollDelta::PixelDelta(PhysicalPosition::new(0.0, 0.0)),
        phase: TouchPhase::Ended,
    });
    mouse.listen_to_event(&WindowEvent::RedrawRequested);
    assert_eq!(mouse.scroll_delta_lines, 1.0);
    assert_eq!(mouse.scroll_delta_pixels, 20.0);
    assert_eq!(mouse.pinch_delta, 0.125);

    mouse.reset();
    assert_eq!(mouse.scroll_delta_lines, 0.0);
    assert_eq!(mouse.scroll_delta_pixels, 0.0);
    assert_eq!(mouse.pinch_delta, 0.0);
}

#[test]
fn nonfinite_pinch_events_do_not_poison_input() {
    let mut mouse = Mouse::new();
    for delta in [0.25, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        mouse.listen_to_event(&WindowEvent::PinchGesture {
            device_id: DeviceId::dummy(),
            delta,
            phase: TouchPhase::Moved,
        });
    }
    assert_eq!(mouse.pinch_delta, 0.25);
}
