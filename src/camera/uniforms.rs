use crate::camera::Camera;
use crate::transform::Transformable;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Default, Debug, Copy, Clone, Pod, Zeroable)]
pub struct CameraUniforms {
    proj_view: [[f32; 4]; 4],
    position: [f32; 3],
    _padding: u32,
}

impl CameraUniforms {
    pub fn update(&mut self, camera: &Camera) {
        self.proj_view = camera.projection_view().as_mat4().to_cols_array_2d();
        self.position = camera.transform().position.as_vec3().to_array();
    }
}
