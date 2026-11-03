use crate::core::renderer::Renderer;
use crate::core::wgpu_context::WGPUContext;
use crate::materials::material::Material;
use crate::materials::shader::{PipelineConfig, Shader};
use crate::materials::utils::create_buffer;
use crate::texture::Texture;
use bytemuck::cast_slice;
use std::rc::Rc;
use wgpu::RenderPass;

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

    material_pipeline: Option<Shader>,

    blinn_phong_uniforms: BlinnPhongUniforms,
    blinn_phong_uniforms_buffer: wgpu::Buffer,

    diffuse_texture: Rc<Texture>,
    ambient_texture: Rc<Texture>,
    specular_texture: Rc<Texture>,
    normal_map: Rc<Texture>,

    pipeline_config: PipelineConfig,
}

impl BlinnPhongMaterial {
    pub fn new(name: &str, wgpu_context: &WGPUContext) -> Self {
        let phong_uniforms = BlinnPhongUniforms::default();
        let phong_uniforms_buffer =
            create_buffer::<BlinnPhongUniforms>(&format!("{} Phong Buffer", name), wgpu_context);

        let diffuse_texture = wgpu_context.empty_texture();
        let ambient_texture = wgpu_context.empty_texture();
        let specular_texture = wgpu_context.empty_texture();
        let normal_map = wgpu_context.empty_texture();

        BlinnPhongMaterial {
            name: name.to_string(),
            material_pipeline: None,

            blinn_phong_uniforms: phong_uniforms,
            blinn_phong_uniforms_buffer: phong_uniforms_buffer,

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

    pub fn compile(&mut self, renderer: &Renderer, wgpu_context: &WGPUContext) {
        self.material_pipeline = Some(Shader::new(
            &format!("{} MaterialPipeline", self.name),
            self.pipeline_config,
            &self.blinn_phong_uniforms_buffer,
            &[
                &self.diffuse_texture,
                &self.ambient_texture,
                &self.specular_texture,
                &self.normal_map,
            ],
            renderer,
            wgpu_context,
        ));
    }

    pub fn set_diffuse_texture(&mut self, texture: Rc<Texture>) {
        self.diffuse_texture = texture;
        self.blinn_phong_uniforms.has_diffuse_texture = 1;
    }

    pub fn set_diffuse_color(&mut self, r: f32, g: f32, b: f32) {
        self.blinn_phong_uniforms.diffuse_color = [r, g, b];
    }

    pub fn set_ambient_texture(&mut self, texture: Rc<Texture>) {
        self.ambient_texture = texture;
        self.blinn_phong_uniforms.has_ambient_texture = 1;
    }

    pub fn set_ambient_color(&mut self, r: f32, g: f32, b: f32) {
        self.blinn_phong_uniforms.ambient_color = [r, g, b];
    }

    pub fn set_specular_texture(&mut self, texture: Rc<Texture>) {
        self.specular_texture = texture;
        self.blinn_phong_uniforms.has_specular_texture = 1;
    }

    pub fn set_specular_color(&mut self, r: f32, g: f32, b: f32) {
        self.blinn_phong_uniforms.specular_color = [r, g, b];
    }

    pub fn set_normal_map(&mut self, texture: Rc<Texture>) {
        self.normal_map = texture;
        self.blinn_phong_uniforms.has_normal_map = 1;
    }

    pub fn set_polygon_mode(&mut self, polygon_mode: wgpu::PolygonMode) {
        self.pipeline_config.polygon_mode = polygon_mode;
    }

    pub fn set_back_face_culling(&mut self, back_face_culling: bool) {
        self.pipeline_config.back_face_culling = back_face_culling;
    }
}

impl Material for BlinnPhongMaterial {
    fn bind(
        &mut self,
        render_pass: &mut RenderPass<'_>,
        renderer: &mut Renderer,
        ctx: &WGPUContext,
    ) {
        if self.material_pipeline.is_none() {
            self.compile(renderer, ctx);
        }
        ctx.queue.write_buffer(
            &self.blinn_phong_uniforms_buffer,
            0,
            cast_slice(&[self.blinn_phong_uniforms]),
        );
        self.material_pipeline.as_ref().unwrap().bind(render_pass);
    }
}
