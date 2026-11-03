use crate::input::transform_control::TransformMouseController;
use cgmath::{Deg, EuclideanSpace, Matrix4, Point3, SquareMatrix, Vector3, perspective};
use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

use crate::core::engine::OPENGL_TO_WGPU_MATRIX;
use crate::input::mouse::Mouse;
use crate::transform::{Transform, Transformable};

pub struct Camera {
    transform: Rc<RefCell<Transform>>,
    pub aspect_ratio: f32,
    pub fov: f32,
    pub z_near: f32,
    pub z_far: f32,

    pub control: Option<Box<dyn TransformMouseController>>,
}

impl Camera {
    pub fn new(aspect_ratio: f32) -> Camera {
        Camera {
            transform: Rc::new(RefCell::new(Transform::new())),
            aspect_ratio,
            fov: 70.0,
            z_near: 0.1,
            z_far: 100.0,
            control: None,
        }
    }
    pub fn view_matrix(&self) -> Matrix4<f32> {
        let mut view = Matrix4::look_at_rh(
            Point3::from_vec(self.transform.borrow().position),
            Point3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
        );
        if let Some(parent) = &self.transform().parent {
            view = view * parent.borrow().compute_world_matrix().invert().unwrap();
        }
        view
    }
    pub fn projection_matrix(&self) -> Matrix4<f32> {
        OPENGL_TO_WGPU_MATRIX
            * perspective(Deg(self.fov), self.aspect_ratio, self.z_near, self.z_far)
    }
    pub fn projection_view(&self) -> Matrix4<f32> {
        self.projection_matrix() * self.view_matrix()
    }

    pub fn listen_to_control(&mut self, mouse: &Mouse, delta_seconds: f32) {
        if let Some(control) = &mut self.control {
            let (position, rotation, scaling) = control.update(mouse, delta_seconds);
            self.transform.borrow_mut().position = position;
            self.transform.borrow_mut().rotation = rotation;
            self.transform.borrow_mut().scaling = scaling;
        }
    }
}

impl Transformable for Camera {
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
