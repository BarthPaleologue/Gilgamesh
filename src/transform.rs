use glam::{DMat4, DQuat, DVec3};
use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

/// A rigid transform composed of a translation and a rotation.
#[derive(Debug)]
#[must_use]
pub struct Transform {
    /// Position in the parent's coordinate space.
    pub position: DVec3,
    /// Local orientation as a unit quaternion.
    pub rotation: DQuat,

    pub parent: Option<Rc<RefCell<Transform>>>,
}

#[repr(C)]
#[derive(Default, Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TransformUniforms {
    pub position: [f32; 3],
    // Due to uniforms requiring 16 byte (4 float) spacing, we need to use a padding field here
    _padding1: u32,
    pub world_matrix: [[f32; 4]; 4],
}

impl TransformUniforms {
    pub fn update(&mut self, transform: &Transform) {
        let world_matrix = transform.world_matrix();
        self.position = world_matrix.w_axis.truncate().as_vec3().to_array();
        self.world_matrix = world_matrix.as_mat4().to_cols_array_2d();
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

            parent: None,
        }
    }
}

impl Transform {
    pub fn new() -> Transform {
        Transform::default()
    }

    /// Orients local -Z toward `target`, using `up` to determine the roll.
    /// Both arguments are in the parent's coordinate space, or world space without a parent.
    /// `target` must differ from `position`; `up` must be normalized and not parallel
    /// to the viewing direction. Position and parent are preserved.
    pub fn look_at(&mut self, target: DVec3, up: DVec3) {
        let back = (self.position - target).normalize();
        let right = up.cross(back).normalize();
        let corrected_up = back.cross(right);

        self.rotation = DQuat::from_rotation_axes(right, corrected_up, back);
    }

    #[must_use]
    pub fn world_matrix(&self) -> DMat4 {
        let mut world_matrix = DMat4::from_rotation_translation(self.rotation, self.position);

        if let Some(parent) = &self.parent {
            let parent_world_matrix = parent.borrow().world_matrix();
            world_matrix = parent_world_matrix * world_matrix;
        }

        world_matrix
    }

    #[must_use]
    pub fn world_position(&self) -> DVec3 {
        if let Some(parent) = &self.parent {
            parent
                .borrow()
                .world_matrix()
                .transform_point3(self.position)
        } else {
            self.position
        }
    }

    #[must_use]
    pub fn local_direction_to_world(&self, local_direction: DVec3) -> DVec3 {
        (self.rotation * local_direction).normalize()
    }

    /// Local -Z expressed in the parent's coordinate space.
    #[must_use]
    pub fn forward(&self) -> DVec3 {
        self.local_direction_to_world(DVec3::NEG_Z)
    }

    #[must_use]
    pub fn right(&self) -> DVec3 {
        self.local_direction_to_world(DVec3::X)
    }

    #[must_use]
    pub fn up(&self) -> DVec3 {
        self.local_direction_to_world(DVec3::Y)
    }
}
