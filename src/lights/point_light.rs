use crate::lights::light::Light;
use crate::photometry::Candela;
use crate::transform::{Transform, Transformable};
use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

pub struct PointLight {
    color: [f32; 3],
    intensity: Candela,
    transform: Rc<RefCell<Transform>>,
}

#[repr(C)]
#[derive(Default, Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PointLightUniforms {
    color: [f32; 3],
    // Due to uniforms requiring 16 byte (4 float) spacing, we need to use a padding field here
    _padding1: u32,
    position: [f32; 3],
    // Due to uniforms requiring 16 byte (4 float) spacing, we need to use a padding field here
    //_padding2: u32,
    intensity: f32,
}

impl PointLightUniforms {
    pub fn update(&mut self, point_light: &PointLight) {
        self.color = point_light.color;
        self.intensity = point_light.intensity.value();
        self.position = point_light
            .transform()
            .world_position()
            .as_vec3()
            .to_array();
    }
}

impl Default for PointLight {
    fn default() -> Self {
        PointLight {
            color: [1.0, 1.0, 1.0],
            intensity: Candela::new(1.0),
            transform: Rc::new(RefCell::new(Transform::new())),
        }
    }
}

impl Light for PointLight {
    fn color(&self) -> [f32; 3] {
        self.color
    }
    fn set_color(&mut self, r: f32, g: f32, b: f32) {
        self.color = [r, g, b];
    }
}

impl PointLight {
    #[must_use]
    pub fn intensity(&self) -> Candela {
        self.intensity
    }
    pub fn set_intensity(&mut self, intensity: Candela) {
        self.intensity = intensity;
    }
}

impl Transformable for PointLight {
    fn transform(&self) -> Ref<'_, Transform> {
        self.transform.borrow()
    }
    fn transform_mut(&self) -> RefMut<'_, Transform> {
        self.transform.borrow_mut()
    }

    fn transform_rc(&self) -> Rc<RefCell<Transform>> {
        self.transform.clone()
    }
}
