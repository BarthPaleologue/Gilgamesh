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

pub struct BlinnPhongMaterial {
    name: String,

    prepared: OnceCell<PreparedMaterial>,

    diffuse_color: [f32; 3],
    ambient_color: [f32; 3],
    specular_color: [f32; 3],

    diffuse_texture: Option<Rc<dyn SampledTexture>>,
    ambient_texture: Option<Rc<dyn SampledTexture>>,
    specular_texture: Option<Rc<Texture<Linear>>>,
    normal_map: Option<Rc<Texture<Linear>>>,

    pipeline_config: PipelineConfig,
}

impl BlinnPhongMaterial {
    pub fn new(name: &str) -> Self {
        BlinnPhongMaterial {
            name: name.to_string(),
            prepared: OnceCell::new(),

            diffuse_color: [1.0, 1.0, 1.0],
            ambient_color: [0.0, 0.0, 0.0],
            specular_color: [1.0, 1.0, 1.0],

            diffuse_texture: None,
            ambient_texture: None,
            specular_texture: None,
            normal_map: None,

            pipeline_config: PipelineConfig {
                shader_source: include_str!("../shader/blinn_phong.wgsl"),
                polygon_mode: wgpu::PolygonMode::Fill,
                back_face_culling: true,
            },
        }
    }

    pub fn set_diffuse_texture<Encoding: ColorEncoding>(&mut self, texture: Rc<Texture<Encoding>>) {
        self.prepared.take();
        self.diffuse_texture = Some(texture);
    }

    pub fn set_diffuse_color(&mut self, r: f32, g: f32, b: f32) {
        self.prepared.take();
        self.diffuse_color = [r, g, b];
    }

    pub fn set_ambient_texture<Encoding: ColorEncoding>(&mut self, texture: Rc<Texture<Encoding>>) {
        self.prepared.take();
        self.ambient_texture = Some(texture);
    }

    pub fn set_ambient_color(&mut self, r: f32, g: f32, b: f32) {
        self.prepared.take();
        self.ambient_color = [r, g, b];
    }

    /// ```compile_fail,E0308
    /// use gilgamesh::materials::blinn_phong::BlinnPhongMaterial;
    /// use gilgamesh::texture::{Srgb, Texture};
    /// use std::rc::Rc;
    ///
    /// fn bind(material: &mut BlinnPhongMaterial, srgb: Rc<Texture<Srgb>>) {
    ///     material.set_specular_texture(srgb);
    /// }
    /// ```
    pub fn set_specular_texture(&mut self, texture: Rc<Texture<Linear>>) {
        self.prepared.take();
        self.specular_texture = Some(texture);
    }

    pub fn set_specular_color(&mut self, r: f32, g: f32, b: f32) {
        self.prepared.take();
        self.specular_color = [r, g, b];
    }

    /// ```compile_fail,E0308
    /// use gilgamesh::materials::blinn_phong::BlinnPhongMaterial;
    /// use gilgamesh::texture::{Srgb, Texture};
    /// use std::rc::Rc;
    ///
    /// fn bind(material: &mut BlinnPhongMaterial, srgb: Rc<Texture<Srgb>>) {
    ///     material.set_normal_map(srgb);
    /// }
    /// ```
    pub fn set_normal_map(&mut self, texture: Rc<Texture<Linear>>) {
        self.prepared.take();
        self.normal_map = Some(texture);
    }

    pub fn set_polygon_mode(&mut self, polygon_mode: wgpu::PolygonMode) {
        self.prepared.take();
        self.pipeline_config.polygon_mode = polygon_mode;
    }

    pub fn set_back_face_culling(&mut self, back_face_culling: bool) {
        self.prepared.take();
        self.pipeline_config.back_face_culling = back_face_culling;
    }

    fn uniforms(&self) -> BlinnPhongUniforms {
        BlinnPhongUniforms {
            diffuse_color: self.diffuse_color,
            has_diffuse_texture: u32::from(self.diffuse_texture.is_some()),
            ambient_color: self.ambient_color,
            has_ambient_texture: u32::from(self.ambient_texture.is_some()),
            specular_color: self.specular_color,
            has_specular_texture: u32::from(self.specular_texture.is_some()),
            has_normal_map: u32::from(self.normal_map.is_some()),
            _padding: [0; 3],
        }
    }
}

impl Material for BlinnPhongMaterial {
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
                    self.specular_texture
                        .as_deref()
                        .map(|texture| texture as &dyn SampledTexture),
                    self.normal_map
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
    use super::BlinnPhongMaterial;

    #[test]
    fn constant_colors_can_be_configured_without_texture_resources() {
        let mut material = BlinnPhongMaterial::new("Constant");
        assert!(material.diffuse_texture.is_none());
        assert!(material.ambient_texture.is_none());
        assert!(material.specular_texture.is_none());
        assert!(material.normal_map.is_none());
        material.set_diffuse_color(0.2, 0.3, 0.4);
        material.set_ambient_color(0.1, 0.0, 0.2);
        material.set_specular_color(0.8, 0.7, 0.6);

        let uniforms = material.uniforms();
        assert_eq!(uniforms.diffuse_color, [0.2, 0.3, 0.4]);
        assert_eq!(uniforms.ambient_color, [0.1, 0.0, 0.2]);
        assert_eq!(uniforms.specular_color, [0.8, 0.7, 0.6]);
        assert_eq!(
            [
                uniforms.has_diffuse_texture,
                uniforms.has_ambient_texture,
                uniforms.has_specular_texture,
                uniforms.has_normal_map,
            ],
            [0; 4],
        );
    }
}
