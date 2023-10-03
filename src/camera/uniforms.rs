use bytemuck::{Pod, Zeroable};
use crate::camera::camera::Camera;
use crate::transform::Transformable;

#[repr(C)]
#[derive(Default, Debug, Copy, Clone, Pod, Zeroable)]
pub struct CameraUniforms {
    proj_view: [[f32; 4]; 4],
    position: [f32; 3],
    _padding: u32,
}

impl CameraUniforms {
    pub fn update(&mut self, camera: &Camera) {
        self.proj_view = camera.projection_view().into();
        self.position = camera.transform().position.into();
    }
}