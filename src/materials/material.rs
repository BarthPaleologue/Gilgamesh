use crate::camera::Camera;
use crate::core::wgpu_context::WGPUContext;
use crate::lights::directional_light::DirectionalLight;
use crate::lights::point_light::PointLight;
use crate::transform::Transform;
use std::cell::Ref;
use wgpu::RenderPass;

pub trait Material {
    fn bind<'a>(
        &'a mut self,
        render_pass: &mut RenderPass<'a>,
        transform: Ref<Transform>,
        active_camera: &Camera,
        point_lights: &[PointLight],
        directional_light: &DirectionalLight,
        wgpu_context: &WGPUContext,
    );
}
