use cgmath::{Deg, EuclideanSpace, Matrix4, perspective, Point3, Vector3};
use crate::input::transform_control::TransformMouseController;

use crate::transform::{Transform, Transformable};
use crate::core::engine::{Engine, OPENGL_TO_WGPU_MATRIX};
use crate::input::mouse::Mouse;

pub struct BasicCamera {
    pub transform: Transform,
    pub aspect_ratio: f32,
    pub fov: f32,
    pub z_near: f32,
    pub z_far: f32,

    pub control: Option<Box<dyn TransformMouseController>>,
}

impl BasicCamera {
    pub fn new(engine: &Engine) -> BasicCamera {
        BasicCamera {
            transform: Transform::new(),
            aspect_ratio: engine.window.inner_size().width as f32 / engine.window.inner_size().height as f32,
            fov: 80.0,
            z_near: 0.1,
            z_far: 100.0,
            control: None,
        }
    }
    pub fn get_view_matrix(&self) -> Matrix4<f32> {
        Matrix4::look_at_rh(Point3::from_vec(self.transform.position), Point3::new(0.0, 0.0, 0.0), Vector3::new(0.0, 1.0, 0.0))
    }
    pub fn get_projection_matrix(&self) -> Matrix4<f32> {
        OPENGL_TO_WGPU_MATRIX * perspective(Deg(self.fov), self.aspect_ratio, self.z_near, self.z_far)
    }

    pub fn listen_to_control(&mut self, mouse: &Mouse, engine: &Engine) {
        if let Some(control) = &mut self.control {
            let (position, rotation, scaling) = control.update(mouse, engine);
            self.transform.position = position;
            self.transform.rotation = rotation;
            self.transform.scaling = scaling;
        }
    }
}

impl Transformable for BasicCamera {
    fn transform(&mut self) -> &mut Transform {
        &mut self.transform
    }
}