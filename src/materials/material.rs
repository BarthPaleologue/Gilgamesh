use std::cell::Ref;
use wgpu::RenderPass;
use crate::camera::camera::Camera;
use crate::core::wgpu_context::WGPUContext;
use crate::lights::directional_light::DirectionalLight;
use crate::lights::point_light::PointLight;
use crate::transform::Transform;

pub trait Material {
    fn bind<'a, 'b>(&'a mut self, render_pass: &'b mut RenderPass<'a>, transform: Ref<Transform>, active_camera: &Camera, point_lights: &[PointLight], directional_light: &DirectionalLight, wgpu_context: &WGPUContext);
}