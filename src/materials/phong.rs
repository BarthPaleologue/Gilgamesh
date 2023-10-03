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
    has_diffuse_texture: u32,
    ambient_color: [f32; 3],
    has_ambient_texture: u32,
    specular_color: [f32; 3],
    has_specular_texture: u32,
    has_normal_map: u32,
}

impl Default for PhongUniforms {
    fn default() -> Self {
        PhongUniforms {
            diffuse_color: [1.0, 1.0, 1.0],
            has_diffuse_texture: 0,
            ambient_color: [0.0, 0.0, 0.0],
            has_ambient_texture: 0,
            specular_color: [1.0, 1.0, 1.0],
            has_specular_texture: 0,
            has_normal_map: 0,
        }
    }
}

pub struct PhongMaterial {
    pub material_pipeline: Option<MaterialPipeline>,

    pub light_uniforms: DirectionalLightUniform,
    pub light_uniforms_buffer: wgpu::Buffer,

    pub point_light_uniforms: [PointLightUniforms; MAX_POINT_LIGHTS],
    pub point_light_buffer: wgpu::Buffer,
    pub nb_point_lights_buffer: wgpu::Buffer,

    pub phong_uniforms: PhongUniforms,
    pub phong_uniforms_buffer: wgpu::Buffer,

    pub diffuse_texture: Texture,
    pub ambient_texture: Texture,
    pub specular_texture: Texture,
    pub normal_map: Texture,
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

        let diffuse_texture = Texture::new_empty("Default Diffuse Texture", wgpu_context);
        let ambient_texture = Texture::new_empty("Default Ambient Texture", wgpu_context);
        let specular_texture = Texture::new_empty("Default Specular Texture", wgpu_context);
        let normal_map = Texture::new_empty("Default Normal Map", wgpu_context);

        PhongMaterial {
            material_pipeline: None,

            light_uniforms,
            light_uniforms_buffer,

            point_light_uniforms,
            point_light_buffer,
            nb_point_lights_buffer,

            phong_uniforms,
            phong_uniforms_buffer,

            diffuse_texture,
            ambient_texture,
            specular_texture,
            normal_map,
        }
    }

    pub fn compile(&mut self, wgpu_context: &mut WGPUContext) {
        self.material_pipeline = Some(MaterialPipeline::new("../shader/phong.wgsl", &vec![
            &self.light_uniforms_buffer,
            &self.point_light_buffer,
            &self.nb_point_lights_buffer,
            &self.phong_uniforms_buffer,
        ], &vec![
            &self.diffuse_texture,
            &self.ambient_texture,
            &self.specular_texture,
            &self.normal_map,
        ], wgpu_context));
    }

    pub fn bind<'a, 'b>(&'a mut self, render_pass: &'b mut RenderPass<'a>, transform: Ref<Transform>, active_camera: &Camera, point_lights: &[PointLight], directional_light: &DirectionalLight, wgpu_context: &mut WGPUContext) {
        if self.material_pipeline.is_none() {
            self.compile(wgpu_context);
        }

        self.light_uniforms.update(directional_light);
        wgpu_context.queue.write_buffer(&self.light_uniforms_buffer, 0, cast_slice(&[self.light_uniforms]));

        for i in 0..point_lights.len() {
            self.point_light_uniforms[i].update(&point_lights[i]);
        }
        wgpu_context.queue.write_buffer(&self.point_light_buffer, 0, cast_slice(&[self.point_light_uniforms]));
        wgpu_context.queue.write_buffer(&self.nb_point_lights_buffer, 0, cast_slice(&[point_lights.len() as u32]));
        wgpu_context.queue.write_buffer(&self.phong_uniforms_buffer, 0, cast_slice(&[self.phong_uniforms]));

        self.material_pipeline.as_mut().unwrap().bind(render_pass, transform, active_camera, wgpu_context);
    }

    pub fn set_diffuse_texture(&mut self, path: &str, wgpu_context: &mut WGPUContext) {
        self.diffuse_texture = Texture::new("Diffuse Texture", path, wgpu_context);
        self.phong_uniforms.has_diffuse_texture = 1;
    }

    pub fn set_diffuse_color(&mut self, r: f32, g: f32, b: f32) {
        self.phong_uniforms.diffuse_color = [r, g, b];
    }

    pub fn set_ambient_texture(&mut self, path: &str, wgpu_context: &mut WGPUContext) {
        self.ambient_texture = Texture::new("Ambient Texture", path, wgpu_context);
        self.phong_uniforms.has_ambient_texture = 1;
    }

    pub fn set_ambient_color(&mut self, r: f32, g: f32, b: f32) {
        self.phong_uniforms.ambient_color = [r, g, b];
    }

    pub fn set_specular_texture(&mut self, path: &str, wgpu_context: &mut WGPUContext) {
        self.specular_texture = Texture::new("Specular Texture", path, wgpu_context);
        self.phong_uniforms.has_specular_texture = 1;
    }

    pub fn set_specular_color(&mut self, r: f32, g: f32, b: f32) {
        self.phong_uniforms.specular_color = [r, g, b];
    }

    pub fn set_normal_map(&mut self, path: &str, wgpu_context: &mut WGPUContext) {
        self.normal_map = Texture::new("Normal Map", path, wgpu_context);
        self.phong_uniforms.has_normal_map = 1;
    }
}