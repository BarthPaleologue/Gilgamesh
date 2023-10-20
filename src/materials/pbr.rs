use std::cell::Ref;
use std::rc::Rc;
use bytemuck::cast_slice;
use wgpu::RenderPass;
use crate::camera::camera::Camera;
use crate::core::wgpu_context::WGPUContext;
use crate::lights::directional_light::{DirectionalLight, DirectionalLightUniform};
use crate::lights::point_light::{PointLight, PointLightUniforms};
use crate::materials::shader::Shader;
use crate::materials::utils::{create_array_buffer, create_buffer};
use crate::settings::MAX_POINT_LIGHTS;
use crate::texture::Texture;
use crate::transform::Transform;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PbrUniforms {
    albedo_color: [f32; 3],
    has_albedo_texture: u32,

    ambient_color: [f32; 3],
    has_ambient_texture: u32,

    metallic: f32,
    has_metallic_texture: u32,
    roughness: f32,
    has_roughness_texture: u32,
}

impl Default for PbrUniforms {
    fn default() -> Self {
        PbrUniforms {
            albedo_color: [1.0, 1.0, 1.0],
            has_albedo_texture: 0,
            ambient_color: [0.0, 0.0, 0.0],
            has_ambient_texture: 0,
            metallic: 0.2,
            has_metallic_texture: 0,
            roughness: 0.8,
            has_roughness_texture: 0,
        }
    }
}

pub struct PbrMaterial {
    name: String,

    material_pipeline: Option<Shader>,

    directional_light_uniforms: DirectionalLightUniform,
    directional_light_uniforms_buffer: wgpu::Buffer,

    point_light_uniforms: [PointLightUniforms; MAX_POINT_LIGHTS],
    point_light_buffer: wgpu::Buffer,
    nb_point_lights_buffer: wgpu::Buffer,

    pbr_uniforms: PbrUniforms,
    pbr_uniforms_buffer: wgpu::Buffer,

    diffuse_texture: Rc<Texture>,
    ambient_texture: Rc<Texture>,
    specular_texture: Rc<Texture>,
    normal_map: Rc<Texture>,

    polygon_mode: wgpu::PolygonMode,
    back_face_culling: bool,
}

impl PbrMaterial {
    pub fn new(name: &str, wgpu_context: &WGPUContext) -> Self {
        let light_uniforms = DirectionalLightUniform::default();
        let light_uniforms_buffer = create_buffer::<DirectionalLightUniform>(&format!("{} DirectionalLight Buffer", name), wgpu_context);

        let point_light_uniforms = [PointLightUniforms::default(); MAX_POINT_LIGHTS];
        let point_light_buffer = create_array_buffer::<PointLightUniforms>(&format!("{} PointLights Array Buffer", name), MAX_POINT_LIGHTS, wgpu_context);

        let nb_point_lights_buffer = create_buffer::<u32>(&format!("{} Number of Point Lights Buffer", name), wgpu_context);

        let phong_uniforms = PbrUniforms::default();
        let phong_uniforms_buffer = create_buffer::<PbrUniforms>(&format!("{} Phong Buffer", name), wgpu_context);

        let diffuse_texture = wgpu_context.empty_texture();
        let ambient_texture = wgpu_context.empty_texture();
        let specular_texture = wgpu_context.empty_texture();
        let normal_map = wgpu_context.empty_texture();

        PbrMaterial {
            name: name.to_string(),
            material_pipeline: None,

            directional_light_uniforms: light_uniforms,
            directional_light_uniforms_buffer: light_uniforms_buffer,

            point_light_uniforms,
            point_light_buffer,
            nb_point_lights_buffer,

            pbr_uniforms: phong_uniforms,
            pbr_uniforms_buffer: phong_uniforms_buffer,

            diffuse_texture,
            ambient_texture,
            specular_texture,
            normal_map,

            polygon_mode: wgpu::PolygonMode::Fill,
            back_face_culling: true,
        }
    }

    pub fn compile(&mut self, wgpu_context: &WGPUContext) {
        self.material_pipeline = Some(Shader::new(&format!("{} MaterialPipeline", self.name), "../shader/blinn_phong.wgsl", &[
            &self.directional_light_uniforms_buffer,
            &self.point_light_buffer,
            &self.nb_point_lights_buffer,
            &self.pbr_uniforms_buffer,
        ], &[
            &self.diffuse_texture,
            &self.ambient_texture,
            &self.specular_texture,
            &self.normal_map,
        ], self.polygon_mode, self.back_face_culling, wgpu_context));
    }

    pub fn bind<'a, 'b>(&'a mut self, render_pass: &'b mut RenderPass<'a>, transform: Ref<Transform>, active_camera: &Camera, point_lights: &[PointLight], directional_light: &DirectionalLight, wgpu_context: &WGPUContext) {
        if self.material_pipeline.is_none() {
            self.compile(wgpu_context);
        }

        self.directional_light_uniforms.update(directional_light);
        wgpu_context.queue.write_buffer(&self.directional_light_uniforms_buffer, 0, cast_slice(&[self.directional_light_uniforms]));

        for i in 0..point_lights.len() {
            self.point_light_uniforms[i].update(&point_lights[i]);
        }
        wgpu_context.queue.write_buffer(&self.point_light_buffer, 0, cast_slice(&[self.point_light_uniforms]));
        wgpu_context.queue.write_buffer(&self.nb_point_lights_buffer, 0, cast_slice(&[point_lights.len() as u32]));
        wgpu_context.queue.write_buffer(&self.pbr_uniforms_buffer, 0, cast_slice(&[self.pbr_uniforms]));

        self.material_pipeline.as_mut().unwrap().bind(render_pass, transform, active_camera, wgpu_context);
    }

    pub fn set_albedo_texture(&mut self, texture: Rc<Texture>) {
        self.diffuse_texture = texture;
        self.pbr_uniforms.has_albedo_texture = 1;
    }

    pub fn set_albedo_color(&mut self, r: f32, g: f32, b: f32) {
        self.pbr_uniforms.albedo_color = [r, g, b];
    }

    pub fn set_ambient_texture(&mut self, texture: Rc<Texture>) {
        self.ambient_texture = texture;
        self.pbr_uniforms.has_ambient_texture = 1;
    }

    pub fn set_ambient_color(&mut self, r: f32, g: f32, b: f32) {
        self.pbr_uniforms.ambient_color = [r, g, b];
    }

    pub fn set_polygon_mode(&mut self, polygon_mode: wgpu::PolygonMode) {
        self.polygon_mode = polygon_mode;
    }

    pub fn set_back_face_culling(&mut self, back_face_culling: bool) {
        self.back_face_culling = back_face_culling;
    }
}