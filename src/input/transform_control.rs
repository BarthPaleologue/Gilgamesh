use cgmath::{Point3, Vector3};
use crate::animation::utils::move_towards;
use crate::core::engine::Engine;
use crate::input::mouse::Mouse;

pub trait TransformMouseController {
    fn update(&mut self, mouse: &Mouse, engine: &Engine) -> (Vector3<f32>, Point3<f32>, Point3<f32>);
}

pub struct OrbitControl {
    target_radius: f32,
    radius: f32,
    phi: f32,
    theta: f32,
    epsilon: f32,
}

impl Default for OrbitControl {
    fn default() -> Self {
        OrbitControl {
            target_radius: 10.0,
            radius: 10.0,
            phi: 0.0,
            theta: -std::f32::consts::PI / 2.0,
            epsilon: 0.0001,
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
    fn update(&mut self, mouse: &Mouse, engine: &Engine) -> (Vector3<f32>, Point3<f32>, Point3<f32>) {
        if mouse.left_button_pressed {
            self.phi += mouse.delta[0] as f32 * engine.get_delta_time();
            self.theta += mouse.delta[1] as f32 * engine.get_delta_time();

            self.theta = self.theta.max(-std::f32::consts::PI + self.epsilon).min(-self.epsilon);
        }

        self.target_radius -= mouse.scroll_delta * engine.get_delta_time() * 30.0;
        self.target_radius = self.target_radius.max(1.0);

        self.radius = move_towards(self.radius, self.target_radius, engine.get_delta_time() * 10.0);

        let x = self.radius * self.theta.sin() * self.phi.cos();
        let y = self.radius * self.theta.cos();
        let z = self.radius * self.theta.sin() * self.phi.sin();

        (Vector3::new(x, y, z), Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 1.0, 1.0))
    }
}