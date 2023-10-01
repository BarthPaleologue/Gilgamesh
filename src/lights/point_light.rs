use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;
use crate::lights::light::Light;
use crate::transform::{Transform, Transformable};

pub struct PointLight {
    color: [f32; 3],
    intensity: f32,
    transform: Rc<RefCell<Transform>>,
}

impl Default for PointLight {
    fn default() -> Self {
        PointLight {
            color: [1.0, 1.0, 1.0],
            intensity: 1.0,
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
    fn intensity(&self) -> f32 {
        self.intensity
    }
    fn set_intensity(&mut self, intensity: f32) {
        self.intensity = intensity;
    }
}

impl Transformable for PointLight {
    fn transform(&self) -> Ref<Transform> {
        self.transform.borrow()
    }
    fn transform_mut(&self) -> RefMut<Transform> {
        self.transform.borrow_mut()
    }

    fn transform_rc(&self) -> Rc<RefCell<Transform>> {
        self.transform.clone()
    }
}