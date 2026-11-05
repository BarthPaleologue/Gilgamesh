use crate::photometry::Nits;
use crate::texture::{ColorTexture, Linear, Texture};
use std::rc::Rc;

/// Metallic-roughness PBR surface properties, shared independently of any renderer.
#[derive(Clone)]
#[must_use]
pub struct PbrMaterial {
    pub name: String,

    pub albedo_color: [f32; 3],
    /// Linear RGB multiplier of the white-reference luminance and optional texture.
    pub emissive_color: [f32; 3],
    pub emissive_luminance: Nits,
    pub metallic: f32,
    pub roughness: f32,

    pub albedo_texture: Option<ColorTexture>,
    pub emissive_texture: Option<ColorTexture>,
    pub normal_map: Option<Rc<Texture<Linear>>>,
    /// Linear roughness sampled from the red channel; replaces the scalar when present.
    pub roughness_map: Option<Rc<Texture<Linear>>>,

    pub double_sided: bool,
}

impl PbrMaterial {
    pub fn new(name: &str) -> Self {
        PbrMaterial {
            name: name.to_string(),

            albedo_color: [1.0, 1.0, 1.0],
            emissive_color: [1.0, 1.0, 1.0],
            emissive_luminance: Nits::new(0.0),
            metallic: 0.0,
            roughness: 0.4,

            albedo_texture: None,
            emissive_texture: None,
            normal_map: None,
            roughness_map: None,

            double_sided: false,
        }
    }

    pub fn set_albedo_texture(&mut self, texture: impl Into<ColorTexture>) {
        self.albedo_texture = Some(texture.into());
    }

    pub fn set_albedo_color(&mut self, r: f32, g: f32, b: f32) {
        self.albedo_color = [r, g, b];
    }

    pub fn set_emissive_texture(&mut self, texture: impl Into<ColorTexture>) {
        self.emissive_texture = Some(texture.into());
    }

    pub fn set_emissive_luminance(&mut self, luminance: Nits) {
        self.emissive_luminance = luminance;
    }

    pub fn set_emissive_color(&mut self, r: f32, g: f32, b: f32) {
        self.emissive_color = [r, g, b];
    }

    pub fn set_normal_map(&mut self, texture: Rc<Texture<Linear>>) {
        self.normal_map = Some(texture);
    }

    /// ```compile_fail,E0308
    /// use gilgamesh::materials::PbrMaterial;
    /// use gilgamesh::texture::{Srgb, Texture};
    /// use std::rc::Rc;
    /// fn bind(material: &mut PbrMaterial, srgb: Rc<Texture<Srgb>>) {
    ///     material.set_roughness_map(srgb);
    /// }
    /// ```
    pub fn set_roughness_map(&mut self, texture: Rc<Texture<Linear>>) {
        self.roughness_map = Some(texture);
    }

    pub fn set_double_sided(&mut self, double_sided: bool) {
        self.double_sided = double_sided;
    }
}

impl super::Material for PbrMaterial {
    fn name(&self) -> &str {
        &self.name
    }

    fn raster(&self) -> super::RasterMaterial<'_> {
        use crate::texture::SampledTexture;

        let mut raster =
            super::RasterMaterial::new(include_str!("../shader/pbr.wgsl"), PbrUniforms::from(self));
        raster.double_sided = self.double_sided;
        raster.textures = vec![
            self.albedo_texture
                .as_ref()
                .map(|t| t as &dyn SampledTexture),
            self.emissive_texture
                .as_ref()
                .map(|t| t as &dyn SampledTexture),
            self.normal_map.as_deref().map(|t| t as &dyn SampledTexture),
            None,
            self.roughness_map
                .as_deref()
                .map(|t| t as &dyn SampledTexture),
        ];
        raster
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct PbrUniforms {
    albedo_color: [f32; 3],
    has_albedo_texture: u32,

    emissive_color: [f32; 3],
    has_emissive_texture: u32,

    metallic: f32,
    has_metallic_texture: u32,
    roughness: f32,
    has_roughness_texture: u32,

    has_normal_map: u32,
    _padding: [u32; 3],
}

impl From<&PbrMaterial> for PbrUniforms {
    fn from(material: &PbrMaterial) -> Self {
        PbrUniforms {
            albedo_color: material.albedo_color,
            has_albedo_texture: u32::from(material.albedo_texture.is_some()),
            emissive_color: material
                .emissive_color
                .map(|c| c * material.emissive_luminance.value()),
            has_emissive_texture: u32::from(material.emissive_texture.is_some()),
            metallic: material.metallic,
            has_metallic_texture: 0,
            roughness: material.roughness,
            has_roughness_texture: u32::from(material.roughness_map.is_some()),
            has_normal_map: u32::from(material.normal_map.is_some()),
            _padding: [0; 3],
        }
    }
}

#[cfg(test)]
mod pbr_tests {
    use super::{PbrMaterial, PbrUniforms};
    use crate::photometry::Nits;

    #[test]
    fn constant_colors_can_be_configured_without_texture_resources() {
        let mut material = PbrMaterial::new("Constant");
        assert!(material.albedo_texture.is_none());
        assert!(material.emissive_texture.is_none());
        assert_eq!(PbrUniforms::from(&material).emissive_color, [0.0; 3]);
        assert!(material.normal_map.is_none());
        assert!(material.roughness_map.is_none());
        material.set_albedo_color(0.2, 0.3, 0.4);
        material.set_emissive_luminance(Nits::new(2.0));
        material.set_emissive_color(0.1, 0.0, 0.2);

        let uniforms = PbrUniforms::from(&material);
        assert_eq!(uniforms.albedo_color, [0.2, 0.3, 0.4]);
        assert_eq!(uniforms.emissive_color, [0.2, 0.0, 0.4]);
        assert_eq!(uniforms.metallic, 0.0);
        assert_eq!(uniforms.roughness, 0.4);
        assert_eq!(
            [
                uniforms.has_albedo_texture,
                uniforms.has_emissive_texture,
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
        assert_eq!(std::mem::offset_of!(PbrUniforms, emissive_color), 16);
        assert_eq!(std::mem::offset_of!(PbrUniforms, metallic), 32);
        assert_eq!(std::mem::offset_of!(PbrUniforms, roughness), 40);
    }
}
