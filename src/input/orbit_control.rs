use crate::camera::{Camera, CameraProjection};
use crate::input::mouse::Mouse;
use crate::transform::Transformable;
use glam::{DVec3, dcamera::rh};

const MIN_RADIUS: f64 = 1.0;
const POLE_EPSILON: f64 = 0.0001;
const ROTATION_RADIANS_PER_PIXEL: f64 = 0.005;
const ZOOM_LOG_SCALE_PER_LINE: f64 = 0.1;
const ZOOM_SMOOTHING_PER_SECOND: f64 = 10.0;
// Scroll pixels and wheel lines have no universal conversion; this defines orbit sensitivity.
const PIXELS_PER_SCROLL_LINE: f64 = 40.0;

pub struct OrbitControl {
    target_radius: f64,
    radius: f64,
    phi: f64,
    theta: f64,
}

impl Default for OrbitControl {
    fn default() -> Self {
        Self {
            target_radius: 10.0,
            radius: 10.0,
            phi: 0.0,
            theta: -std::f64::consts::PI / 4.0,
        }
    }
}

impl OrbitControl {
    pub fn set_radius(&mut self, radius: f64) {
        self.target_radius = radius.max(MIN_RADIUS);
        self.radius = self.target_radius;
    }

    /// Orbits around the origin in the camera's parent coordinate space.
    /// Changes position and orientation, leaving projection untouched.
    pub fn update<ProjectionType: CameraProjection>(
        &mut self,
        camera: &mut Camera<ProjectionType>,
        mouse: &Mouse,
        delta_seconds: f64,
    ) {
        if mouse.left_button_pressed {
            self.phi += mouse.delta[0] * ROTATION_RADIANS_PER_PIXEL;
            self.theta += mouse.delta[1] * ROTATION_RADIANS_PER_PIXEL;
            self.theta = self
                .theta
                .clamp(-std::f64::consts::PI + POLE_EPSILON, -POLE_EPSILON);
        }

        let scroll_lines =
            mouse.scroll_delta_lines + mouse.scroll_delta_pixels / PIXELS_PER_SCROLL_LINE;
        let zoom = scroll_lines * ZOOM_LOG_SCALE_PER_LINE + mouse.pinch_delta;
        self.target_radius = (self.target_radius * (-zoom).exp()).max(MIN_RADIUS);
        let smoothing = -(-ZOOM_SMOOTHING_PER_SECOND * delta_seconds).exp_m1();
        self.radius += (self.target_radius - self.radius) * smoothing;

        let position = DVec3::new(
            self.radius * self.theta.sin() * self.phi.cos(),
            self.radius * self.theta.cos(),
            self.radius * self.theta.sin() * self.phi.sin(),
        );
        let rotation = rh::view::look_at_quat(position, DVec3::ZERO, DVec3::Y).conjugate();
        let mut transform = camera.transform_mut();
        transform.position = position;
        transform.rotation = rotation;
    }
}
