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

    prepared: OnceCell<PreparedMaterial>,

    pbr_uniforms: PbrUniforms,

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

        let diffuse_texture = wgpu_context.empty_texture();
        let ambient_texture = wgpu_context.empty_texture();
        let normal_map = wgpu_context.empty_texture();
        let metallic_texture = wgpu_context.empty_texture();
        let roughness_texture = wgpu_context.empty_texture();

        PbrMaterial {
            name: name.to_string(),
            prepared: OnceCell::new(),

            pbr_uniforms,

            diffuse_texture,
            ambient_texture,
            normal_map,
            metallic_texture,
            roughness_texture,

            pipeline_config: PipelineConfig {
                shader_source: include_str!("../shader/pbr.wgsl"),
                polygon_mode: wgpu::PolygonMode::Fill,
                back_face_culling: true,
            },
        }
    }

    pub fn set_albedo_texture(&mut self, texture: Rc<Texture>) {
        self.prepared.take();
        self.diffuse_texture = texture;
        self.pbr_uniforms.has_albedo_texture = 1;
    }

    pub fn set_albedo_color(&mut self, r: f32, g: f32, b: f32) {
        self.prepared.take();
        self.pbr_uniforms.albedo_color = [r, g, b];
    }

    pub fn set_ambient_texture(&mut self, texture: Rc<Texture>) {
        self.prepared.take();
        self.ambient_texture = texture;
        self.pbr_uniforms.has_ambient_texture = 1;
    }

    pub fn set_ambient_color(&mut self, r: f32, g: f32, b: f32) {
        self.prepared.take();
        self.pbr_uniforms.ambient_color = [r, g, b];
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

impl Material for PbrMaterial {
    fn prepare(&self, renderer: &mut Renderer, ctx: &WGPUContext) -> &PreparedMaterial {
        self.prepared.get_or_init(|| {
            let buffer = ctx
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&self.name),
                    contents: bytemuck::bytes_of(&self.pbr_uniforms),
                    usage: wgpu::BufferUsages::UNIFORM,
                });
            PreparedMaterial::new(
                &self.name,
                self.pipeline_config,
                &buffer,
                &[
                    &self.diffuse_texture,
                    &self.ambient_texture,
                    &self.normal_map,
                    &self.metallic_texture,
                    &self.roughness_texture,
                ],
                renderer,
                ctx,
            )
        })
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
