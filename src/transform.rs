use cgmath::Point3;

#[derive(Copy, Clone, Debug)]
pub struct Transform {
    pub position: Point3<f32>,
    pub rotation: Point3<f32>,
}

impl Transform {
    pub fn new() -> Transform {
        Transform {
            position: Point3::new(0.0, 0.0, 0.0),
            rotation: Point3::new(0.0, 0.0, 0.0),
        }
    }
    pub fn set_position(&mut self, x: f32, y: f32, z: f32) {
        self.position.x = x;
        self.position.y = y;
        self.position.z = z;
    }
}