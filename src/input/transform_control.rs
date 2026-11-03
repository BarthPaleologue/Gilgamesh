use crate::animation::utils::move_towards;
use crate::input::mouse::Mouse;
use glam::Vec3;

pub trait TransformMouseController {
    fn update(&mut self, mouse: &Mouse, delta_seconds: f32) -> (Vec3, Vec3, Vec3);
}

pub struct OrbitControl {
    target_radius: f32,
    radius: f32,
    phi: f32,
    theta: f32,
    epsilon: f32,
    rotation_speed: f32,
}

impl Default for OrbitControl {
    fn default() -> Self {
        OrbitControl {
            target_radius: 10.0,
            radius: 10.0,
            phi: 0.0,
            theta: -std::f32::consts::PI / 4.0,
            epsilon: 0.0001,
            rotation_speed: 0.5,
        }
    }
}

impl OrbitControl {
    pub fn set_radius(&mut self, radius: f32) {
        self.target_radius = radius;
        self.radius = radius;
    }
}

impl TransformMouseController for OrbitControl {
    fn update(&mut self, mouse: &Mouse, delta_seconds: f32) -> (Vec3, Vec3, Vec3) {
        if mouse.left_button_pressed {
            self.phi += mouse.delta[0] as f32 * self.rotation_speed * delta_seconds;
            self.theta += mouse.delta[1] as f32 * self.rotation_speed * delta_seconds;

            self.theta = self
                .theta
                .max(-std::f32::consts::PI + self.epsilon)
                .min(-self.epsilon);
        }

        self.target_radius -= mouse.scroll_delta * delta_seconds * 30.0;
        self.target_radius = self.target_radius.max(1.0);

        self.radius = move_towards(self.radius, self.target_radius, delta_seconds * 10.0);

        let x = self.radius * self.theta.sin() * self.phi.cos();
        let y = self.radius * self.theta.cos();
        let z = self.radius * self.theta.sin() * self.phi.sin();

        (Vec3::new(x, y, z), Vec3::ZERO, Vec3::ONE)
    }
}
