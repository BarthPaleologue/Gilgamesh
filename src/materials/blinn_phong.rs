use crate::core::renderer::Renderer;
use crate::core::wgpu_context::WGPUContext;
use crate::materials::material::Material;
use crate::materials::material_pipeline::PipelineConfig;
use crate::materials::prepared_material::PreparedMaterial;
use crate::texture::Texture;
use std::cell::OnceCell;
use std::rc::Rc;
use wgpu::util::DeviceExt;

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

    prepared: OnceCell<PreparedMaterial>,

    blinn_phong_uniforms: BlinnPhongUniforms,

    diffuse_texture: Rc<Texture>,
    ambient_texture: Rc<Texture>,
    specular_texture: Rc<Texture>,
    normal_map: Rc<Texture>,

    pipeline_config: PipelineConfig,
}

impl BlinnPhongMaterial {
    pub fn new(name: &str, wgpu_context: &WGPUContext) -> Self {
        let phong_uniforms = BlinnPhongUniforms::default();

        let diffuse_texture = wgpu_context.empty_texture();
        let ambient_texture = wgpu_context.empty_texture();
        let specular_texture = wgpu_context.empty_texture();
        let normal_map = wgpu_context.empty_texture();

        BlinnPhongMaterial {
            name: name.to_string(),
            prepared: OnceCell::new(),

            blinn_phong_uniforms: phong_uniforms,

            diffuse_texture,
            ambient_texture,
            specular_texture,
            normal_map,

            pipeline_config: PipelineConfig {
                shader_file: "../shader/blinn_phong.wgsl",
                polygon_mode: wgpu::PolygonMode::Fill,
                back_face_culling: true,
            },
        }
    }

    pub fn set_diffuse_texture(&mut self, texture: Rc<Texture>) {
        self.prepared.take();
        self.diffuse_texture = texture;
        self.blinn_phong_uniforms.has_diffuse_texture = 1;
    }

    pub fn set_diffuse_color(&mut self, r: f32, g: f32, b: f32) {
        self.prepared.take();
        self.blinn_phong_uniforms.diffuse_color = [r, g, b];
    }

    pub fn set_ambient_texture(&mut self, texture: Rc<Texture>) {
        self.prepared.take();
        self.ambient_texture = texture;
        self.blinn_phong_uniforms.has_ambient_texture = 1;
    }

    pub fn set_ambient_color(&mut self, r: f32, g: f32, b: f32) {
        self.prepared.take();
        self.blinn_phong_uniforms.ambient_color = [r, g, b];
    }

    pub fn set_specular_texture(&mut self, texture: Rc<Texture>) {
        self.prepared.take();
        self.specular_texture = texture;
        self.blinn_phong_uniforms.has_specular_texture = 1;
    }

    pub fn set_specular_color(&mut self, r: f32, g: f32, b: f32) {
        self.prepared.take();
        self.blinn_phong_uniforms.specular_color = [r, g, b];
    }

    pub fn set_normal_map(&mut self, texture: Rc<Texture>) {
        self.prepared.take();
        self.normal_map = texture;
        self.blinn_phong_uniforms.has_normal_map = 1;
    }

    pub fn set_polygon_mode(&mut self, polygon_mode: wgpu::PolygonMode) {
        self.prepared.take();
        self.pipeline_config.polygon_mode = polygon_mode;
    }

    pub fn set_back_face_culling(&mut self, back_face_culling: bool) {
        self.prepared.take();
        self.pipeline_config.back_face_culling = back_face_culling;
    }
}

impl Material for BlinnPhongMaterial {
    fn prepare(&self, renderer: &mut Renderer, ctx: &WGPUContext) -> &PreparedMaterial {
        self.prepared.get_or_init(|| {
            let uniforms = self.blinn_phong_uniforms;
            let buffer = ctx
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&self.name),
                    contents: bytemuck::bytes_of(&uniforms),
                    usage: wgpu::BufferUsages::UNIFORM,
                });
            PreparedMaterial::new(
                &self.name,
                self.pipeline_config,
                &buffer,
                &[
                    &self.diffuse_texture,
                    &self.ambient_texture,
                    &self.specular_texture,
                    &self.normal_map,
                ],
                renderer,
                ctx,
            )
        })
    }
}
