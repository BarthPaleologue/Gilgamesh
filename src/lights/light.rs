pub trait Light {
    fn color(&self) -> [f32; 3];
    fn set_color(&mut self, r: f32, g: f32, b: f32);
    fn intensity(&self) -> f32;
    fn set_intensity(&mut self, intensity: f32);
}