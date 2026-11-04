use crate::core::renderer::Renderer;
use crate::core::wgpu_context::WGPUContext;
use crate::materials::material::Material;
use crate::materials::material_pipeline::PipelineConfig;
use crate::materials::prepared_material::PreparedMaterial;
use crate::texture::{ColorEncoding, Linear, SampledTexture, Texture};
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

pub struct PbrMaterial {
    name: String,

    prepared: OnceCell<PreparedMaterial>,

    albedo_color: [f32; 3],
    ambient_color: [f32; 3],
    metallic: f32,
    roughness: f32,

    diffuse_texture: Option<Rc<dyn SampledTexture>>,
    ambient_texture: Option<Rc<dyn SampledTexture>>,
    normal_map: Option<Rc<Texture<Linear>>>,
    metallic_texture: Option<Rc<Texture<Linear>>>,
    roughness_texture: Option<Rc<Texture<Linear>>>,

    pipeline_config: PipelineConfig,
}

impl PbrMaterial {
    pub fn new(name: &str) -> Self {
        PbrMaterial {
            name: name.to_string(),
            prepared: OnceCell::new(),

            albedo_color: [1.0, 1.0, 1.0],
            ambient_color: [0.0, 0.0, 0.0],
            metallic: 0.7,
            roughness: 0.4,

            diffuse_texture: None,
            ambient_texture: None,
            normal_map: None,
            metallic_texture: None,
            roughness_texture: None,

            pipeline_config: PipelineConfig {
                shader_source: include_str!("../shader/pbr.wgsl"),
                polygon_mode: wgpu::PolygonMode::Fill,
                back_face_culling: true,
            },
        }
    }

    pub fn set_albedo_texture<Encoding: ColorEncoding>(&mut self, texture: Rc<Texture<Encoding>>) {
        self.prepared.take();
        self.diffuse_texture = Some(texture);
    }

    pub fn set_albedo_color(&mut self, r: f32, g: f32, b: f32) {
        self.prepared.take();
        self.albedo_color = [r, g, b];
    }

    pub fn set_ambient_texture<Encoding: ColorEncoding>(&mut self, texture: Rc<Texture<Encoding>>) {
        self.prepared.take();
        self.ambient_texture = Some(texture);
    }

    pub fn set_ambient_color(&mut self, r: f32, g: f32, b: f32) {
        self.prepared.take();
        self.ambient_color = [r, g, b];
    }

    pub fn set_polygon_mode(&mut self, polygon_mode: wgpu::PolygonMode) {
        self.prepared.take();
        self.pipeline_config.polygon_mode = polygon_mode;
    }

    pub fn set_back_face_culling(&mut self, back_face_culling: bool) {
        self.prepared.take();
        self.pipeline_config.back_face_culling = back_face_culling;
    }

    fn uniforms(&self) -> PbrUniforms {
        PbrUniforms {
            albedo_color: self.albedo_color,
            has_albedo_texture: u32::from(self.diffuse_texture.is_some()),
            ambient_color: self.ambient_color,
            has_ambient_texture: u32::from(self.ambient_texture.is_some()),
            metallic: self.metallic,
            has_metallic_texture: u32::from(self.metallic_texture.is_some()),
            roughness: self.roughness,
            has_roughness_texture: u32::from(self.roughness_texture.is_some()),
            has_normal_map: u32::from(self.normal_map.is_some()),
            _padding: [0; 3],
        }
    }
}

impl Material for PbrMaterial {
    fn prepare(&self, renderer: &mut Renderer, ctx: &WGPUContext) -> &PreparedMaterial {
        self.prepared.get_or_init(|| {
            let uniforms = self.uniforms();
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
                    self.diffuse_texture.as_deref(),
                    self.ambient_texture.as_deref(),
                    self.normal_map
                        .as_deref()
                        .map(|texture| texture as &dyn SampledTexture),
                    self.metallic_texture
                        .as_deref()
                        .map(|texture| texture as &dyn SampledTexture),
                    self.roughness_texture
                        .as_deref()
                        .map(|texture| texture as &dyn SampledTexture),
                ],
                renderer,
                ctx,
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{PbrMaterial, PbrUniforms};

    #[test]
    fn constant_colors_can_be_configured_without_texture_resources() {
        let mut material = PbrMaterial::new("Constant");
        assert!(material.diffuse_texture.is_none());
        assert!(material.ambient_texture.is_none());
        assert!(material.normal_map.is_none());
        assert!(material.metallic_texture.is_none());
        assert!(material.roughness_texture.is_none());
        material.set_albedo_color(0.2, 0.3, 0.4);
        material.set_ambient_color(0.1, 0.0, 0.2);

        let uniforms = material.uniforms();
        assert_eq!(uniforms.albedo_color, [0.2, 0.3, 0.4]);
        assert_eq!(uniforms.ambient_color, [0.1, 0.0, 0.2]);
        assert_eq!(uniforms.metallic, 0.7);
        assert_eq!(uniforms.roughness, 0.4);
        assert_eq!(
            [
                uniforms.has_albedo_texture,
                uniforms.has_ambient_texture,
                uniforms.has_normal_map,
                uniforms.has_metallic_texture,
                uniforms.has_roughness_texture,
            ],
            [0; 5],
        );
    }

    #[test]
    fn gpu_uniforms_match_shader_layout() {
        assert_eq!(std::mem::size_of::<PbrUniforms>(), 64);
        assert_eq!(std::mem::offset_of!(PbrUniforms, ambient_color), 16);
        assert_eq!(std::mem::offset_of!(PbrUniforms, metallic), 32);
        assert_eq!(std::mem::offset_of!(PbrUniforms, roughness), 40);
    }
}
