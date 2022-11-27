use std::f32::consts::PI;
use cgmath::{Matrix4, perspective, Point3, Rad, Vector3};

#[path = "./transform.rs"]
mod transform;

use transform::{Transform};
use crate::transforms::OPENGL_TO_WGPU_MATRIX;

#[derive(Copy, Clone)]
pub struct BasicCamera {
    pub transform: Transform,
}

impl BasicCamera {
    pub fn new() -> BasicCamera {
        BasicCamera {
            transform: Transform::new()
        }
    }
    pub fn get_view_matrix(&self) -> Matrix4<f32> {
        Matrix4::look_at_rh(self.transform.position, Point3::new(0.0, 0.0, 0.0), Vector3::new(0.0, 1.0, 0.0))
    }
    pub fn get_projection_matrix(&self) -> Matrix4<f32> {
        OPENGL_TO_WGPU_MATRIX * perspective(Rad(2.0 * PI / 5.0), 16.0 / 9.0, 0.1, 100.0)
    }
}

pub struct FreeCamera {
    pub basic_camera: BasicCamera,
}

impl FreeCamera {
    pub fn new() -> FreeCamera {
        FreeCamera {
            basic_camera: BasicCamera::new()
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
