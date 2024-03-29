use cgmath::{InnerSpace, Matrix4, Point3, Rad, Vector3, Vector4};

#[derive(Debug)]
pub struct Transform {
    pub position: Vector3<f32>,
    pub rotation: Point3<f32>,
    pub scaling: Point3<f32>,
}

impl Default for Transform {
    fn default() -> Self {
        Transform {
            position: Vector3::new(0.0, 0.0, 0.0),
            rotation: Point3::new(0.0, 0.0, 0.0),
            scaling: Point3::new(1.0, 1.0, 1.0),
        }
    }
}

impl Transform {
    pub fn new() -> Transform {
        Transform::default()
    }

    pub fn set_position(&mut self, x: f32, y: f32, z: f32) {
        self.position.x = x;
        self.position.y = y;
        self.position.z = z;
    }
    pub fn compute_world_matrix(&self) -> Matrix4<f32> {
        let position = Matrix4::from_translation(self.position);
        let rotation_x = Matrix4::from_angle_x(Rad(self.rotation.x));
        let rotation_y = Matrix4::from_angle_y(Rad(self.rotation.y));
        let rotation_z = Matrix4::from_angle_z(Rad(self.rotation.z));
        let scaling = Matrix4::from_nonuniform_scale(self.scaling.x, self.scaling.y, self.scaling.z);

        position * scaling * rotation_z * rotation_y * rotation_x
    }

    pub fn forward(&self) -> Vector3<f32> {
        let rotation_x = Matrix4::from_angle_x(Rad(self.rotation.x));
        let rotation_y = Matrix4::from_angle_y(Rad(self.rotation.y));
        let rotation_z = Matrix4::from_angle_z(Rad(self.rotation.z));
        let rotation = rotation_z * rotation_y * rotation_x;
        let forward4 = rotation * Vector4::new(0.0, 0.0, 1.0, 1.0);
        let forward = Vector3::new(forward4.x, forward4.y, forward4.z);
        forward.normalize()
    }

    pub fn right(&self) -> Vector3<f32> {
        let rotation_x = Matrix4::from_angle_x(Rad(self.rotation.x));
        let rotation_y = Matrix4::from_angle_y(Rad(self.rotation.y));
        let rotation_z = Matrix4::from_angle_z(Rad(self.rotation.z));
        let rotation = rotation_z * rotation_y * rotation_x;
        let right4 = rotation * Vector4::new(1.0, 0.0, 0.0, 1.0);
        let right = Vector3::new(right4.x, right4.y, right4.z);
        right.normalize()
    }
}