use winit::event::{ElementState, MouseScrollDelta, WindowEvent};

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
        self.new_position = match event {
            WindowEvent::CursorMoved { position: pos, .. } => [pos.x, pos.y],
            _ => self.new_position,
        };

        self.delta = [
            self.new_position[0] - self.position[0],
            self.new_position[1] - self.position[1],
        ];

        match event {
            WindowEvent::MouseWheel {
                delta: MouseScrollDelta::LineDelta(_, y),
                ..
            } => {
                self.scroll_delta_lines += f64::from(*y);
            }
            WindowEvent::MouseWheel {
                delta: MouseScrollDelta::PixelDelta(delta),
                ..
            } => {
                self.scroll_delta_pixels += delta.y;
            }
            WindowEvent::PinchGesture { delta, .. } if delta.is_finite() => {
                self.pinch_delta += *delta;
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: winit::event::MouseButton::Left,
                ..
            } => {
                self.left_button_pressed = true;
            }
            WindowEvent::MouseInput {
                state: ElementState::Released,
                button: winit::event::MouseButton::Left,
                ..
            } => {
                self.left_button_pressed = false;
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
