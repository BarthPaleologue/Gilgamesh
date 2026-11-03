use glam::{DMat4, DQuat, DVec3};
use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

#[derive(Debug)]
pub struct Transform {
    pub position: DVec3,
    /// Local orientation as a unit quaternion.
    pub rotation: DQuat,
    pub scaling: DVec3,

    pub parent: Option<Rc<RefCell<Transform>>>,
}

#[repr(C)]
#[derive(Default, Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TransformUniforms {
    pub position: [f32; 3],
    // Due to uniforms requiring 16 byte (4 float) spacing, we need to use a padding field here
    _padding1: u32,
    pub world_matrix: [[f32; 4]; 4],
    pub normal_matrix: [[f32; 4]; 4],
}

impl TransformUniforms {
    pub fn update(&mut self, transform: &Transform) {
        self.position = transform.position.as_vec3().to_array();
        self.world_matrix = transform
            .compute_world_matrix()
            .as_mat4()
            .to_cols_array_2d();
        self.normal_matrix = transform
            .compute_normal_matrix()
            .as_mat4()
            .to_cols_array_2d();
    }
}

pub trait Transformable {
    fn transform(&self) -> Ref<'_, Transform>;
    fn transform_mut(&self) -> RefMut<'_, Transform>;
    fn transform_rc(&self) -> Rc<RefCell<Transform>>;
}

impl Default for Transform {
    fn default() -> Self {
        Transform {
            position: DVec3::ZERO,
            rotation: DQuat::IDENTITY,
            scaling: DVec3::ONE,

            parent: None,
        }
    }
}

impl Transform {
    pub fn new() -> Transform {
        Transform::default()
    }

    pub fn compute_world_matrix(&self) -> DMat4 {
        let position = DMat4::from_translation(self.position);
        let rotation = DMat4::from_quat(self.rotation);
        let scaling = DMat4::from_scale(self.scaling);

        let mut world_matrix = position * scaling * rotation;

        if let Some(parent) = &self.parent {
            let parent_world_matrix = parent.borrow().compute_world_matrix();
            world_matrix = parent_world_matrix * world_matrix;
        }

        world_matrix
    }

    pub fn compute_normal_matrix(&self) -> DMat4 {
        let world_matrix = self.compute_world_matrix();
        world_matrix
            .try_inverse()
            .expect("Transform matrix is not invertible")
            .transpose()
    }

    pub fn local_direction_to_world(&self, local_direction: DVec3) -> DVec3 {
        (self.rotation * local_direction).normalize()
    }

    pub fn forward(&self) -> DVec3 {
        self.local_direction_to_world(DVec3::Z)
    }

    pub fn right(&self) -> DVec3 {
        self.local_direction_to_world(DVec3::X)
    }

    pub fn up(&self) -> DVec3 {
        self.local_direction_to_world(DVec3::Y)
    }
}
