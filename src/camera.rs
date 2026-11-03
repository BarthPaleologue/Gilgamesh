pub mod uniforms;

use glam::{DMat4, dcamera::rh};
use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

use crate::transform::{Transform, Transformable};

pub struct Camera {
    transform: Rc<RefCell<Transform>>,
    pub aspect_ratio: f64,
    /// Vertical field of view in degrees.
    pub fov: f64,
    pub z_near: f64,
    pub z_far: f64,
}

impl Camera {
    pub fn new(aspect_ratio: f64) -> Camera {
        Camera {
            transform: Rc::new(RefCell::new(Transform::new())),
            aspect_ratio,
            fov: 70.0,
            z_near: 0.1,
            z_far: 100.0,
        }
    }
    /// Inverts the camera's world transform, with local -Z as the viewing direction.
    pub fn view_matrix(&self) -> DMat4 {
        self.transform()
            .compute_world_matrix()
            .try_inverse()
            .expect("Camera world matrix is not invertible")
    }
    pub fn projection_matrix(&self) -> DMat4 {
        rh::proj::directx::perspective(
            self.fov.to_radians(),
            self.aspect_ratio,
            self.z_near,
            self.z_far,
        )
    }
    pub fn projection_view(&self) -> DMat4 {
        self.projection_matrix() * self.view_matrix()
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
