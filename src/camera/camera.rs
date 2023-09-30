use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;
use cgmath::{Deg, EuclideanSpace, Matrix4, perspective, Point3, Vector3};
use crate::input::transform_control::TransformMouseController;

use crate::transform::{Transform, Transformable};
use crate::core::engine::{Engine, OPENGL_TO_WGPU_MATRIX};
use crate::input::mouse::Mouse;

pub struct Camera {
    transform: Rc<RefCell<Transform>>,
    pub aspect_ratio: f32,
    pub fov: f32,
    pub z_near: f32,
    pub z_far: f32,

    pub control: Option<Box<dyn TransformMouseController>>,
}

impl Camera {
    pub fn new(engine: &Engine) -> Camera {
        Camera {
            transform: Rc::new(RefCell::new(Transform::new())),
            aspect_ratio: engine.window.inner_size().width as f32 / engine.window.inner_size().height as f32,
            fov: 70.0,
            z_near: 0.1,
            z_far: 100.0,
            control: None,
        }
    }
    pub fn view_matrix(&self) -> Matrix4<f32> {
        Matrix4::look_at_rh(Point3::from_vec(self.transform.borrow().position), Point3::new(0.0, 0.0, 0.0), Vector3::new(0.0, 1.0, 0.0))
    }
    pub fn projection_matrix(&self) -> Matrix4<f32> {
        OPENGL_TO_WGPU_MATRIX * perspective(Deg(self.fov), self.aspect_ratio, self.z_near, self.z_far)
    }
    pub fn view_projection_matrix(&self) -> Matrix4<f32> {
        self.view_matrix() * self.projection_matrix()
    }

    pub fn listen_to_control(&mut self, mouse: &Mouse, engine: &Engine) {
        if let Some(control) = &mut self.control {
            let (position, rotation, scaling) = control.update(mouse, engine);
            self.transform.borrow_mut().position = position;
            self.transform.borrow_mut().rotation = rotation;
            self.transform.borrow_mut().scaling = scaling;
        }
    }
}

impl Transformable for Camera {
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