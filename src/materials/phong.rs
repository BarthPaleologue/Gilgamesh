use std::cell::Ref;
use bytemuck::cast_slice;
use wgpu::RenderPass;
use crate::camera::camera::Camera;
use crate::core::wgpu_context::WGPUContext;
use crate::lights::directional_light::{DirectionalLight, DirectionalLightUniform};
use crate::lights::point_light::{PointLight, PointLightUniforms};
use crate::materials::material_pipeline::MaterialPipeline;
use crate::materials::utils::{create_array_buffer, create_buffer};
use crate::settings::MAX_POINT_LIGHTS;
use crate::texture::Texture;
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
            specular_color: [1.0, 1.0, 1.0],
            _padding3: 0,
        }
    }
}

pub struct PhongMaterial {
    pub material_pipeline: MaterialPipeline,

    pub light_uniforms: DirectionalLightUniform,
    pub light_uniforms_buffer: wgpu::Buffer,

    pub point_light_uniforms: [PointLightUniforms; MAX_POINT_LIGHTS],
    pub point_light_buffer: wgpu::Buffer,
    pub nb_point_lights_buffer: wgpu::Buffer,

    pub phong_uniforms: PhongUniforms,
    pub phong_uniforms_buffer: wgpu::Buffer,

    pub diffuse_texture: Texture,
}

impl PhongMaterial {
    pub fn new(wgpu_context: &mut WGPUContext) -> Self {
        let light_uniforms = DirectionalLightUniform::default();
        let light_uniforms_buffer = create_buffer::<DirectionalLightUniform>("DirectionalLight Buffer", wgpu_context);

        let point_light_uniforms = [PointLightUniforms::default(); MAX_POINT_LIGHTS];
        let point_light_buffer = create_array_buffer::<PointLightUniforms>("PointLights Array Buffer", MAX_POINT_LIGHTS, wgpu_context);

        let nb_point_lights_buffer = create_buffer::<u32>("Number of Point Lights Buffer", wgpu_context);

        let phong_uniforms = PhongUniforms::default();
        let phong_uniforms_buffer = create_buffer::<PhongUniforms>("Phong Buffer", wgpu_context);

        let diffuse_texture = Texture::new("test", "textures/test.png", wgpu_context);

        let material_pipeline = MaterialPipeline::new("../shader/phong.wgsl", &vec![
            &light_uniforms_buffer,
            &point_light_buffer,
            &nb_point_lights_buffer,
            &phong_uniforms_buffer,
        ], &vec![
            &diffuse_texture
        ], wgpu_context);

        PhongMaterial {
            material_pipeline,

            diffuse_texture,

            light_uniforms,
            light_uniforms_buffer,

            point_light_uniforms,
            point_light_buffer,
            nb_point_lights_buffer,

            phong_uniforms,
            phong_uniforms_buffer,
        }
    }

    pub fn bind<'a, 'b>(&'a mut self, render_pass: &'b mut RenderPass<'a>, transform: Ref<Transform>, active_camera: &Camera, point_lights: &[PointLight], directional_light: &DirectionalLight, wgpu_context: &mut WGPUContext) {
        self.light_uniforms.update(directional_light);
        wgpu_context.queue.write_buffer(&self.light_uniforms_buffer, 0, cast_slice(&[self.light_uniforms]));

        for i in 0..point_lights.len() {
            self.point_light_uniforms[i].update(&point_lights[i]);
        }
        wgpu_context.queue.write_buffer(&self.point_light_buffer, 0, cast_slice(&[self.point_light_uniforms]));
        wgpu_context.queue.write_buffer(&self.nb_point_lights_buffer, 0, cast_slice(&[point_lights.len() as u32]));
        wgpu_context.queue.write_buffer(&self.phong_uniforms_buffer, 0, cast_slice(&[self.phong_uniforms]));

        self.material_pipeline.bind(render_pass, transform, active_camera, wgpu_context);
    }

    pub fn set_diffuse_color(&mut self, r: f32, g: f32, b: f32) {
        self.phong_uniforms.diffuse_color = [r, g, b];
    }

    pub fn set_ambient_color(&mut self, r: f32, g: f32, b: f32) {
        self.phong_uniforms.ambient_color = [r, g, b];
    }

    pub fn set_specular_color(&mut self, r: f32, g: f32, b: f32) {
        self.phong_uniforms.specular_color = [r, g, b];
    }
}