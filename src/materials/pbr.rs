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
