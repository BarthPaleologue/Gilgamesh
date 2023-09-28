use std::f32::consts::PI;
use cgmath::{EuclideanSpace, Matrix4, perspective, Point3, Rad, Vector3};


use crate::transform::Transform;
use crate::engine::{Engine, OPENGL_TO_WGPU_MATRIX};

pub struct BasicCamera {
    pub transform: Transform,
    pub aspect_ratio: f32,
    pub z_near: f32,
    pub z_far: f32,
}

impl BasicCamera {
    pub fn new(engine: &Engine) -> BasicCamera {
        BasicCamera {
            transform: Transform::new(),
            aspect_ratio: engine.window.inner_size().width as f32 / engine.window.inner_size().height as f32,
            z_near: 0.1,
            z_far: 100.0,
        }
    }
    pub fn get_view_matrix(&self) -> Matrix4<f32> {
        Matrix4::look_at_rh(Point3::from_vec(self.transform.position), Point3::new(0.0, 0.0, 0.0), Vector3::new(0.0, 1.0, 0.0))
    }
    pub fn get_projection_matrix(&self) -> Matrix4<f32> {
        OPENGL_TO_WGPU_MATRIX * perspective(Rad(2.0 * PI / 5.0), self.aspect_ratio, self.z_near, self.z_far)
    }
}

pub struct FreeCamera {
    pub basic_camera: BasicCamera,
}

impl FreeCamera {
    pub fn new(engine: &Engine) -> FreeCamera {
        FreeCamera {
            basic_camera: BasicCamera::new(engine)
        }
    }
}

pub trait Transformable {
    fn tf(&mut self) -> &mut Transform;
}

impl Transformable for FreeCamera {
    fn tf(&mut self) -> &mut Transform {
        &mut self.basic_camera.transform
    }
}
