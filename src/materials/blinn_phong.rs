use crate::photometry::Nits;
use crate::texture::{ColorTexture, Linear, Texture};
use std::rc::Rc;

/// Surface properties, shared independently of any renderer.
#[derive(Clone)]
#[must_use]
pub struct BlinnPhongMaterial {
    pub name: String,

    pub diffuse_color: [f32; 3],
    /// Linear RGB multiplier of the white-reference luminance and optional texture.
    pub ambient_color: [f32; 3],
    pub ambient_luminance: Nits,
    pub specular_color: [f32; 3],

    pub diffuse_texture: Option<ColorTexture>,
    pub ambient_texture: Option<ColorTexture>,
    pub specular_texture: Option<Rc<Texture<Linear>>>,
    pub normal_map: Option<Rc<Texture<Linear>>>,

    pub double_sided: bool,
}

impl BlinnPhongMaterial {
    pub fn new(name: &str) -> Self {
        BlinnPhongMaterial {
            name: name.to_string(),

            diffuse_color: [1.0, 1.0, 1.0],
            ambient_color: [1.0, 1.0, 1.0],
            ambient_luminance: Nits::new(0.0),
            specular_color: [0.04, 0.04, 0.04],

            diffuse_texture: None,
            ambient_texture: None,
            specular_texture: None,
            normal_map: None,

            double_sided: false,
        }
    }

    pub fn set_diffuse_texture(&mut self, texture: impl Into<ColorTexture>) {
        self.diffuse_texture = Some(texture.into());
    }

    pub fn set_diffuse_color(&mut self, r: f32, g: f32, b: f32) {
        self.diffuse_color = [r, g, b];
    }

    pub fn set_ambient_texture(&mut self, texture: impl Into<ColorTexture>) {
        self.ambient_texture = Some(texture.into());
    }

    pub fn set_ambient_luminance(&mut self, luminance: Nits) {
        self.ambient_luminance = luminance;
    }

    pub fn set_ambient_color(&mut self, r: f32, g: f32, b: f32) {
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
        self.specular_texture = Some(texture);
    }

    pub fn set_specular_color(&mut self, r: f32, g: f32, b: f32) {
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
        self.normal_map = Some(texture);
    }

    pub fn set_double_sided(&mut self, double_sided: bool) {
        self.double_sided = double_sided;
    }
}
