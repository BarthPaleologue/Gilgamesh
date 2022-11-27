use crate::{Transform, vertex_data};

pub struct Mesh {
    pub transform: Transform,
    pub positions: Vec<[f32;3]>,
    pub colors: Vec<[f32;3]>,
}

impl Mesh {
    pub fn new() -> Mesh {
        Mesh {
            transform: Transform::new(),
            positions: Vec::new(),
            colors: Vec::new()
        }
    }

    pub fn new_cube() -> Mesh {
        Mesh {
            transform: Transform::new(),
            positions: vertex_data::cube_positions(),
            colors: vertex_data::cube_colors(),
        }
    }
}