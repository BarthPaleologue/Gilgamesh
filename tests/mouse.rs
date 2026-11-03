use gilgamesh::input::mouse::Mouse;
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
    assert_eq!(mouse.scroll_delta, 3.0);

    mouse.reset();
    assert_eq!(mouse.scroll_delta, 0.0);
}
