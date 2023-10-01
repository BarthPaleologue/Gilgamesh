use crate::lights::light::Light;

pub struct DirectionalLight {
    color: [f32; 3],
    intensity: f32,
    direction: [f32; 3],
}

impl Default for DirectionalLight {
    fn default() -> Self {
        DirectionalLight {
            color: [1.0, 1.0, 1.0],
            intensity: 1.0,
            direction: [0.0, 1.0, 0.0],
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
    fn intensity(&self) -> f32 {
        self.intensity
    }
    fn set_intensity(&mut self, intensity: f32) {
        self.intensity = intensity;
    }
}