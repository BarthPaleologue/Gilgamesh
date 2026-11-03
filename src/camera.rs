pub mod uniforms;

use crate::input::transform_control::TransformMouseController;
use glam::{Mat4, camera::rh};
use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

use crate::input::mouse::Mouse;
use crate::transform::{Transform, Transformable};

pub struct Camera {
    transform: Rc<RefCell<Transform>>,
    pub aspect_ratio: f32,
    /// Vertical field of view in degrees.
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
    /// Inverts the camera's world transform, with local -Z as the viewing direction.
    pub fn view_matrix(&self) -> Mat4 {
        self.transform()
            .compute_world_matrix()
            .try_inverse()
            .expect("Camera world matrix is not invertible")
    }
    pub fn projection_matrix(&self) -> Mat4 {
        rh::proj::directx::perspective(
            self.fov.to_radians(),
            self.aspect_ratio,
            self.z_near,
            self.z_far,
        )
    }
    pub fn projection_view(&self) -> Mat4 {
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
