use std::cell::Ref;
use bytemuck::cast_slice;
use wgpu::RenderPass;
use crate::camera::camera::Camera;
use crate::camera::uniforms::CameraUniforms;
use crate::core::wgpu_context::WGPUContext;
use crate::lights::directional_light::{DirectionalLight, DirectionalLightUniform};
use crate::lights::point_light::{PointLight, PointLightUniforms};
use crate::materials::material_pipeline::MaterialPipeline;
use crate::materials::utils::{create_array_buffer, create_buffer};
use crate::settings::MAX_POINT_LIGHTS;
use crate::transform::Transform;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PhongUniforms {
    diffuse_color: [f32; 3],
    _padding1: u32,
    ambient_color: [f32; 3],
    _padding2: u32,
    specular_color: [f32; 3],
    _padding3: u32,
}

impl Default for PhongUniforms {
    fn default() -> Self {
        PhongUniforms {
            diffuse_color: [1.0, 1.0, 1.0],
            _padding1: 0,
            ambient_color: [0.0, 0.0, 0.0],
            _padding2: 0,
            specular_color: [0.0, 0.0, 0.0],
            _padding3: 0,
        }
    }
}

pub struct PhongMaterial {
    pub phong_uniforms: PhongUniforms,
    pub material_pipeline: MaterialPipeline,

    pub camera_uniforms: CameraUniforms,
    pub camera_uniforms_buffer: wgpu::Buffer,

    pub light_uniforms: DirectionalLightUniform,
    pub light_uniforms_buffer: wgpu::Buffer,

    pub point_light_uniforms: [PointLightUniforms; MAX_POINT_LIGHTS],
    pub point_light_buffer: wgpu::Buffer,
    pub nb_point_lights_buffer: wgpu::Buffer,
}

impl PhongMaterial {
    pub fn new(wgpu_context: &mut WGPUContext) -> Self {
        let camera_uniforms = CameraUniforms::default();
        let camera_uniforms_buffer = create_buffer::<CameraUniforms>("Camera Buffer", wgpu_context);

        let light_uniforms = DirectionalLightUniform::default();
        let light_uniforms_buffer = create_buffer::<DirectionalLightUniform>("DirectionalLight Buffer", wgpu_context);

        let point_light_uniforms = [PointLightUniforms::default(); MAX_POINT_LIGHTS];
        let point_light_buffer = create_array_buffer::<PointLightUniforms>("PointLights Array Buffer", MAX_POINT_LIGHTS, wgpu_context);

        let nb_point_lights_buffer = create_buffer::<u32>("Number of Point Lights Buffer", wgpu_context);

        let material_pipeline = MaterialPipeline::new_default(&vec![
            &camera_uniforms_buffer,
            &light_uniforms_buffer,
            &point_light_buffer,
            &nb_point_lights_buffer,
        ], wgpu_context);

        PhongMaterial {
            phong_uniforms: PhongUniforms::default(),
            material_pipeline,

            camera_uniforms,
            camera_uniforms_buffer,

            light_uniforms,
            light_uniforms_buffer,

            point_light_uniforms,
            point_light_buffer,

            nb_point_lights_buffer,
        }
    }

    pub fn bind<'a, 'b>(&'a mut self, render_pass: &'b mut RenderPass<'a>, transform: Ref<Transform>, active_camera: &Camera, point_lights: &[PointLight], directional_light: &DirectionalLight, wgpu_context: &mut WGPUContext) {
        self.camera_uniforms.update(active_camera);
        wgpu_context.queue.write_buffer(&self.camera_uniforms_buffer, 0, cast_slice(&[self.camera_uniforms]));

        self.light_uniforms.update(directional_light);
        wgpu_context.queue.write_buffer(&self.light_uniforms_buffer, 0, cast_slice(&[self.light_uniforms]));

        for i in 0..point_lights.len() {
            self.point_light_uniforms[i].update(&point_lights[i]);
        }
        wgpu_context.queue.write_buffer(&self.point_light_buffer, 0, cast_slice(&[self.point_light_uniforms]));
        wgpu_context.queue.write_buffer(&self.nb_point_lights_buffer, 0, cast_slice(&[point_lights.len() as u32]));

        self.material_pipeline.bind(render_pass, transform, active_camera, wgpu_context);
    }
}