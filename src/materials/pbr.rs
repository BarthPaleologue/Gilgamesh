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
pub struct PbrUniforms {
    albedo_color: [f32; 3],
    has_albedo_texture: u32,

    ambient_color: [f32; 3],
    has_ambient_texture: u32,

    metallic: f32,
    has_metallic_texture: u32,
    roughness: f32,
    has_roughness_texture: u32,

    has_normal_map: u32,
    _padding: [u32; 3],
}

impl Default for PbrUniforms {
    fn default() -> Self {
        Self {
            albedo_color: [1.0, 1.0, 1.0],
            has_albedo_texture: 0,
            ambient_color: [0.0, 0.0, 0.0],
            has_ambient_texture: 0,
            metallic: 0.7,
            has_metallic_texture: 0,
            roughness: 0.4,
            has_roughness_texture: 0,
            has_normal_map: 0,
            _padding: [0; 3],
        }
    }
}

pub struct PbrMaterial {
    name: String,

    material_pipeline: Option<Shader>,

    pbr_uniforms: PbrUniforms,
    pbr_uniforms_buffer: wgpu::Buffer,

    diffuse_texture: Rc<Texture>,
    ambient_texture: Rc<Texture>,
    normal_map: Rc<Texture>,
    metallic_texture: Rc<Texture>,
    roughness_texture: Rc<Texture>,

    pipeline_config: PipelineConfig,
}

impl PbrMaterial {
    pub fn new(name: &str, wgpu_context: &WGPUContext) -> Self {
        let pbr_uniforms = PbrUniforms::default();
        let phong_uniforms_buffer =
            create_buffer::<PbrUniforms>(&format!("{} Phong Buffer", name), wgpu_context);

        let diffuse_texture = wgpu_context.empty_texture();
        let ambient_texture = wgpu_context.empty_texture();
        let normal_map = wgpu_context.empty_texture();
        let metallic_texture = wgpu_context.empty_texture();
        let roughness_texture = wgpu_context.empty_texture();

        PbrMaterial {
            name: name.to_string(),
            material_pipeline: None,

            pbr_uniforms,
            pbr_uniforms_buffer: phong_uniforms_buffer,

            diffuse_texture,
            ambient_texture,
            normal_map,
            metallic_texture,
            roughness_texture,

            pipeline_config: PipelineConfig {
                shader_file: "../shader/pbr.wgsl",
                polygon_mode: wgpu::PolygonMode::Fill,
                back_face_culling: true,
            },
        }
    }

    pub fn compile(&mut self, renderer: &Renderer, wgpu_context: &WGPUContext) {
        self.material_pipeline = Some(Shader::new(
            &format!("{} MaterialPipeline", self.name),
            self.pipeline_config,
            &self.pbr_uniforms_buffer,
            &[
                &self.diffuse_texture,
                &self.ambient_texture,
                &self.normal_map,
                &self.metallic_texture,
                &self.roughness_texture,
            ],
            renderer,
            wgpu_context,
        ));
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
        self.pipeline_config.polygon_mode = polygon_mode;
    }

    pub fn set_back_face_culling(&mut self, back_face_culling: bool) {
        self.pipeline_config.back_face_culling = back_face_culling;
    }
}

impl Material for PbrMaterial {
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
            &self.pbr_uniforms_buffer,
            0,
            cast_slice(&[self.pbr_uniforms]),
        );
        self.material_pipeline.as_ref().unwrap().bind(render_pass);
    }
}

#[cfg(test)]
mod tests {
    use super::PbrUniforms;

    #[test]
    fn gpu_uniforms_match_shader_layout() {
        assert_eq!(std::mem::size_of::<PbrUniforms>(), 64);
        assert_eq!(std::mem::offset_of!(PbrUniforms, ambient_color), 16);
        assert_eq!(std::mem::offset_of!(PbrUniforms, metallic), 32);
        assert_eq!(std::mem::offset_of!(PbrUniforms, roughness), 40);
    }
}
