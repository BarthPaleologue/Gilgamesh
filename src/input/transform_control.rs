use crate::animation::utils::move_towards;
use crate::input::mouse::Mouse;
use glam::{DQuat, DVec3, dcamera::rh};

pub trait TransformMouseController {
    fn update(&mut self, mouse: &Mouse, delta_seconds: f64) -> (DVec3, DQuat, DVec3);
}

pub struct OrbitControl {
    target_radius: f64,
    radius: f64,
    phi: f64,
    theta: f64,
    epsilon: f64,
    rotation_speed: f64,
}

impl Default for OrbitControl {
    fn default() -> Self {
        OrbitControl {
            target_radius: 10.0,
            radius: 10.0,
            phi: 0.0,
            theta: -std::f64::consts::PI / 4.0,
            epsilon: 0.0001,
            rotation_speed: 0.5,
        }
    }
}

impl OrbitControl {
    pub fn set_radius(&mut self, radius: f64) {
        self.target_radius = radius;
        self.radius = radius;
    }
}

impl TransformMouseController for OrbitControl {
    fn update(&mut self, mouse: &Mouse, delta_seconds: f64) -> (DVec3, DQuat, DVec3) {
        if mouse.left_button_pressed {
            self.phi += mouse.delta[0] * self.rotation_speed * delta_seconds;
            self.theta += mouse.delta[1] * self.rotation_speed * delta_seconds;

            self.theta = self
                .theta
                .max(-std::f64::consts::PI + self.epsilon)
                .min(-self.epsilon);
        }

        self.target_radius -= mouse.scroll_delta * delta_seconds * 30.0;
        self.target_radius = self.target_radius.max(1.0);

        self.radius = move_towards(self.radius, self.target_radius, delta_seconds * 10.0);

        let x = self.radius * self.theta.sin() * self.phi.cos();
        let y = self.radius * self.theta.cos();
        let z = self.radius * self.theta.sin() * self.phi.sin();

        let position = DVec3::new(x, y, z);
        let rotation = rh::view::look_at_quat(position, DVec3::ZERO, DVec3::Y).conjugate();
        (position, rotation, DVec3::ONE)
    }
}
