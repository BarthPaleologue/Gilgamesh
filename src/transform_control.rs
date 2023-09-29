use cgmath::{Point3, Vector3};
use crate::mouse::Mouse;

pub trait TransformControl {
    fn update(&mut self, mouse: &Mouse) -> (Vector3<f32>, Point3<f32>, Point3<f32>);
}

pub struct OrbitControl {
    radius: f32,
    phi: f32,
    theta: f32,
    epsilon: f32,
}

impl Default for OrbitControl {
    fn default() -> Self {
        OrbitControl {
            radius: 10.0,
            phi: 0.0,
            theta: std::f32::consts::PI / 2.0,
            epsilon: 0.0001,
        }
    }
}

impl TransformControl for OrbitControl {
    fn update(&mut self, mouse: &Mouse) -> (Vector3<f32>, Point3<f32>, Point3<f32>) {
        if mouse.left_button_pressed {
            self.phi += mouse.delta[0] * 0.01;
            self.theta += mouse.delta[1] * 0.01;

            self.theta = self.theta.max(-std::f32::consts::PI + self.epsilon).min(-self.epsilon);
        }

        let x = self.radius * self.theta.sin() * self.phi.cos();
        let y = self.radius * self.theta.cos();
        let z = self.radius * self.theta.sin() * self.phi.sin();

        (Vector3::new(x, y, z), Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 1.0, 1.0))
    }
}