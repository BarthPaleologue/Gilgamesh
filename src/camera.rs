mod projection;
pub mod uniforms;

pub use projection::{CameraProjection, OffAxis, Orthographic, Perspective, Projection};

use glam::DMat4;
use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

use crate::transform::{Transform, Transformable};

#[derive(Default)]
pub struct Camera<ProjectionType: CameraProjection = Projection> {
    transform: Rc<RefCell<Transform>>,
    pub projection: ProjectionType,
}

impl<ProjectionType: CameraProjection> Camera<ProjectionType> {
    pub fn new(projection: ProjectionType) -> Self {
        Self {
            transform: Rc::new(RefCell::new(Transform::new())),
            projection,
        }
    }

    /// Inverts the camera's world transform, with local -Z as the viewing direction.
    pub fn view_matrix(&self) -> DMat4 {
        self.transform()
            .world_matrix()
            .try_inverse()
            .expect("Camera world matrix is not invertible")
    }
    pub fn projection_matrix(&self) -> DMat4 {
        self.projection.matrix()
    }
    pub fn projection_view(&self) -> DMat4 {
        self.projection_matrix() * self.view_matrix()
    }
}

impl<ProjectionType: CameraProjection> Transformable for Camera<ProjectionType> {
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
