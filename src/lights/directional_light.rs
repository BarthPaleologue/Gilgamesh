use crate::lights::light::Light;
use crate::photometry::Lux;
use bytemuck::{Pod, Zeroable};

pub struct DirectionalLight {
    color: [f32; 3],
    illuminance: Lux,
    direction: [f32; 3],
}

#[repr(C)]
#[derive(Default, Debug, Copy, Clone, Pod, Zeroable)]
pub struct DirectionalLightUniform {
    color: [f32; 3],
    // Due to uniforms requiring 16 byte (4 float) spacing, we need to use a padding field here
    _padding: u32,
    direction: [f32; 3],
    // Due to uniforms requiring 16 byte (4 float) spacing, we need to use a padding field here
    //_padding2: u32,
    intensity: f32,
}

impl DirectionalLightUniform {
    pub fn update(&mut self, light: &DirectionalLight) {
        self.color = light.color;
        self.intensity = light.illuminance.value();
        self.direction = light.direction;
    }
}

impl Default for DirectionalLight {
    fn default() -> Self {
        DirectionalLight {
            color: [1.0, 1.0, 1.0],
            illuminance: Lux::new(1.0),
            direction: [0.707, -0.707, 0.0],
        }
    }
}

impl Light for DirectionalLight {
    fn color(&self) -> [f32; 3] {
        self.color
    }
    fn set_color(&mut self, r: f32, g: f32, b: f32) {
        self.color = [r, g, b];
    }
}

impl DirectionalLight {
    /// Illuminance on a surface perpendicular to the light direction.
    #[must_use]
    pub fn illuminance(&self) -> Lux {
        self.illuminance
    }
    pub fn set_illuminance(&mut self, illuminance: Lux) {
        self.illuminance = illuminance;
    }
}
