use winit::event::{ElementState, MouseScrollDelta, WindowEvent};

#[must_use]
pub struct Mouse {
    pub position: [f64; 2],
    pub new_position: [f64; 2],
    pub delta: [f64; 2],
    pub left_button_pressed: bool,
    pub right_button_pressed: bool,
    pub middle_button_pressed: bool,
    /// Vertical scrolling measured in wheel lines.
    pub scroll_delta_lines: f64,
    /// Vertical scrolling measured in physical pixels, commonly from a trackpad.
    pub scroll_delta_pixels: f64,
    /// Native pinch magnification; positive values zoom in.
    pub pinch_delta: f64,
}

impl Default for Mouse {
    fn default() -> Self {
        Mouse {
            position: [0.0, 0.0],
            new_position: [0.0, 0.0],
            delta: [0.0, 0.0],
            left_button_pressed: false,
            right_button_pressed: false,
            middle_button_pressed: false,
            scroll_delta_lines: 0.0,
            scroll_delta_pixels: 0.0,
            pinch_delta: 0.0,
        }
    }
}

impl Mouse {
    pub fn new() -> Mouse {
        Mouse::default()
    }

    pub fn listen_to_event(&mut self, event: &WindowEvent) {
        self.listen_to_event_with_consumption(event, false);
    }

    pub(crate) fn listen_to_event_with_consumption(&mut self, event: &WindowEvent, consumed: bool) {
        if let WindowEvent::CursorMoved { position, .. } = event {
            self.new_position = [position.x, position.y];
            if consumed {
                self.position = self.new_position;
                self.delta = [0.0; 2];
            } else {
                self.delta = [
                    self.new_position[0] - self.position[0],
                    self.new_position[1] - self.position[1],
                ];
            }
        }

        match event {
            WindowEvent::MouseWheel {
                delta: MouseScrollDelta::LineDelta(_, y),
                ..
            } if !consumed => self.scroll_delta_lines += f64::from(*y),
            WindowEvent::MouseWheel {
                delta: MouseScrollDelta::PixelDelta(delta),
                ..
            } if !consumed => self.scroll_delta_pixels += delta.y,
            WindowEvent::PinchGesture { delta, .. } if !consumed && delta.is_finite() => {
                self.pinch_delta += *delta;
            }
            WindowEvent::MouseInput { state, button, .. }
                if !consumed || *state == ElementState::Released =>
            {
                let pressed = *state == ElementState::Pressed;
                match button {
                    winit::event::MouseButton::Left => self.left_button_pressed = pressed,
                    winit::event::MouseButton::Right => self.right_button_pressed = pressed,
                    winit::event::MouseButton::Middle => self.middle_button_pressed = pressed,
                    _ => {}
                }
            }
            _ => {}
        }
    }

    pub fn reset(&mut self) {
        self.position = self.new_position;
        self.delta = [0.0, 0.0];
        self.scroll_delta_lines = 0.0;
        self.scroll_delta_pixels = 0.0;
        self.pinch_delta = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use winit::dpi::PhysicalPosition;
    use winit::event::{DeviceId, MouseButton, TouchPhase};

    fn button(button: MouseButton, state: ElementState) -> WindowEvent {
        WindowEvent::MouseInput {
            device_id: DeviceId::dummy(),
            state,
            button,
        }
    }

    #[test]
    fn consumed_presses_do_not_start_drags_and_releases_always_end_them() {
        let mut mouse = Mouse::default();
        for button_type in [MouseButton::Left, MouseButton::Right, MouseButton::Middle] {
            let any_pressed = |mouse: &Mouse| {
                mouse.left_button_pressed
                    || mouse.right_button_pressed
                    || mouse.middle_button_pressed
            };
            mouse.listen_to_event_with_consumption(
                &button(button_type, ElementState::Pressed),
                true,
            );
            assert!(!any_pressed(&mouse));
            mouse.listen_to_event(&button(button_type, ElementState::Pressed));
            assert!(any_pressed(&mouse));
            mouse.listen_to_event_with_consumption(
                &button(button_type, ElementState::Released),
                true,
            );
            assert!(!any_pressed(&mouse));
        }
    }

    #[test]
    fn captured_movement_and_zoom_do_not_leak_or_cause_a_cursor_jump() {
        let mut mouse = Mouse::default();
        let movement = |x, y| WindowEvent::CursorMoved {
            device_id: DeviceId::dummy(),
            position: PhysicalPosition::new(x, y),
        };
        mouse.listen_to_event_with_consumption(&movement(200.0, 100.0), true);
        assert_eq!(mouse.delta, [0.0; 2]);
        mouse.listen_to_event(&movement(205.0, 104.0));
        assert_eq!(mouse.delta, [5.0, 4.0]);
        for delta in [
            MouseScrollDelta::LineDelta(0.0, 2.0),
            MouseScrollDelta::PixelDelta(PhysicalPosition::new(0.0, 80.0)),
        ] {
            mouse.listen_to_event_with_consumption(
                &WindowEvent::MouseWheel {
                    device_id: DeviceId::dummy(),
                    delta,
                    phase: TouchPhase::Moved,
                },
                true,
            );
        }
        mouse.listen_to_event_with_consumption(
            &WindowEvent::PinchGesture {
                device_id: DeviceId::dummy(),
                delta: 0.5,
                phase: TouchPhase::Moved,
            },
            true,
        );
        assert_eq!(mouse.delta, [5.0, 4.0]);
        assert_eq!(mouse.scroll_delta_lines, 0.0);
        assert_eq!(mouse.scroll_delta_pixels, 0.0);
        assert_eq!(mouse.pinch_delta, 0.0);
    }
}
