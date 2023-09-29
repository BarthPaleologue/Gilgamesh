use winit::event::{ElementState, WindowEvent};

pub struct Mouse {
    pub position: [f32; 2],
    pub delta: [f32; 2],
    pub left_button_pressed: bool,
    pub right_button_pressed: bool,
    pub middle_button_pressed: bool,
}

impl Default for Mouse {
    fn default() -> Self {
        Mouse {
            position: [0.0, 0.0],
            delta: [0.0, 0.0],
            left_button_pressed: false,
            right_button_pressed: false,
            middle_button_pressed: false,
        }
    }
}

impl Mouse {
    pub fn new() -> Mouse {
        Mouse::default()
    }

    pub fn listen_to_event(&mut self, event: &WindowEvent) {
        let new_position = match event {
            WindowEvent::CursorMoved {
                position: pos,
                ..
            } => {
                [pos.x as f32, pos.y as f32]
            }
            _ => self.position,
        };

        self.delta = [new_position[0] - self.position[0], new_position[1] - self.position[1]];
        self.position = new_position;

        match event {
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
}