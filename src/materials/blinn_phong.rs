use std::cell::Ref;
use std::rc::Rc;
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
pub struct BlinnPhongUniforms {
    diffuse_color: [f32; 3],
    has_diffuse_texture: u32,
    ambient_color: [f32; 3],
    has_ambient_texture: u32,
    specular_color: [f32; 3],
    has_specular_texture: u32,
    has_normal_map: u32,
    _padding: [u32; 3],
}

impl Default for BlinnPhongUniforms {
    fn default() -> Self {
        BlinnPhongUniforms {
            diffuse_color: [1.0, 1.0, 1.0],
            has_diffuse_texture: 0,
            ambient_color: [0.0, 0.0, 0.0],
            has_ambient_texture: 0,
            specular_color: [1.0, 1.0, 1.0],
            has_specular_texture: 0,
            has_normal_map: 0,
            _padding: [0; 3],
        }
    }
}

pub struct BlinnPhongMaterial {
    name: String,

    material_pipeline: Option<MaterialPipeline>,

    light_uniforms: DirectionalLightUniform,
    light_uniforms_buffer: wgpu::Buffer,

    point_light_uniforms: [PointLightUniforms; MAX_POINT_LIGHTS],
    point_light_buffer: wgpu::Buffer,
    nb_point_lights_buffer: wgpu::Buffer,

    phong_uniforms: BlinnPhongUniforms,
    phong_uniforms_buffer: wgpu::Buffer,

    diffuse_texture: Rc<Texture>,
    ambient_texture: Rc<Texture>,
    specular_texture: Rc<Texture>,
    normal_map: Rc<Texture>,

    polygon_mode: wgpu::PolygonMode,
    back_face_culling: bool,
}

impl BlinnPhongMaterial {
    pub fn new(name: &str, wgpu_context: &WGPUContext) -> Self {
        let light_uniforms = DirectionalLightUniform::default();
        let light_uniforms_buffer = create_buffer::<DirectionalLightUniform>(&format!("{} DirectionalLight Buffer", name), wgpu_context);

        let point_light_uniforms = [PointLightUniforms::default(); MAX_POINT_LIGHTS];
        let point_light_buffer = create_array_buffer::<PointLightUniforms>(&format!("{} PointLights Array Buffer", name), MAX_POINT_LIGHTS, wgpu_context);

        let nb_point_lights_buffer = create_buffer::<u32>(&format!("{} Number of Point Lights Buffer", name), wgpu_context);

        let phong_uniforms = BlinnPhongUniforms::default();
        let phong_uniforms_buffer = create_buffer::<BlinnPhongUniforms>(&format!("{} Phong Buffer", name), wgpu_context);

        let diffuse_texture = wgpu_context.empty_texture();
        let ambient_texture = wgpu_context.empty_texture();
        let specular_texture = wgpu_context.empty_texture();
        let normal_map = wgpu_context.empty_texture();

        BlinnPhongMaterial {
            name: name.to_string(),
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

            polygon_mode: wgpu::PolygonMode::Fill,
            back_face_culling: true,
        }
    }

    pub fn compile(&mut self, wgpu_context: &WGPUContext) {
        self.material_pipeline = Some(MaterialPipeline::new(&format!("{} MaterialPipeline", self.name), "../shader/blinn_phong.wgsl", &[
            &self.light_uniforms_buffer,
            &self.point_light_buffer,
            &self.nb_point_lights_buffer,
            &self.phong_uniforms_buffer,
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

    pub fn set_diffuse_texture(&mut self, texture: Rc<Texture>) {
        self.diffuse_texture = texture;
        self.phong_uniforms.has_diffuse_texture = 1;
    }

    pub fn set_diffuse_color(&mut self, r: f32, g: f32, b: f32) {
        self.phong_uniforms.diffuse_color = [r, g, b];
    }

    pub fn set_ambient_texture(&mut self, texture: Rc<Texture>) {
        self.ambient_texture = texture;
        self.phong_uniforms.has_ambient_texture = 1;
    }

    pub fn set_ambient_color(&mut self, r: f32, g: f32, b: f32) {
        self.phong_uniforms.ambient_color = [r, g, b];
    }

    pub fn set_specular_texture(&mut self, texture: Rc<Texture>) {
        self.specular_texture = texture;
        self.phong_uniforms.has_specular_texture = 1;
    }

    pub fn set_specular_color(&mut self, r: f32, g: f32, b: f32) {
        self.phong_uniforms.specular_color = [r, g, b];
    }

    pub fn set_normal_map(&mut self, texture: Rc<Texture>) {
        self.normal_map = texture;
        self.phong_uniforms.has_normal_map = 1;
    }

    pub fn set_polygon_mode(&mut self, polygon_mode: wgpu::PolygonMode) {
        self.polygon_mode = polygon_mode;
    }

    pub fn set_back_face_culling(&mut self, back_face_culling: bool) {
        self.back_face_culling = back_face_culling;
    }
}