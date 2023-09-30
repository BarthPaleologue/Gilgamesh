use bytemuck::{Pod, Zeroable};
use crate::camera::camera::Camera;
use crate::transform::Transformable;

#[repr(C)]
#[derive(Default, Debug, Copy, Clone, Pod, Zeroable)]
pub struct CameraUniforms {
    view_proj: [[f32; 4]; 4],
    position: [f32; 3],
}

impl CameraUniforms {
    pub fn update(&mut self, camera: &Camera) {
        self.view_proj = camera.view_projection_matrix().into();
        self.position = camera.transform().position.into();
    }
}