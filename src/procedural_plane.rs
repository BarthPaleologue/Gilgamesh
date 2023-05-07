use crate::engine::Engine;
use crate::mesh::Mesh;

impl Mesh {
    pub fn new_procedural_plane(size: f32, nb_subdivisions: u32, engine: &mut Engine) -> Mesh {
        let mut positions = vec!([0.0, 0.0, 0.0] ; ((nb_subdivisions * nb_subdivisions) as usize));
        let mut indices = vec!(0 ; ((6 * (nb_subdivisions - 1) * (nb_subdivisions - 1)) as usize));

        for x in 0..nb_subdivisions {
            for y in 0..nb_subdivisions {
                let actual_x = (x as f32 - (nb_subdivisions as f32 / 2.0)) * size / nb_subdivisions as f32;
                let actual_y = (y as f32 - (nb_subdivisions as f32 / 2.0)) * size / nb_subdivisions as f32;

                positions[(x * nb_subdivisions + y) as usize] = [actual_x, 0.0, actual_y];

                if x == nb_subdivisions - 1 || y == nb_subdivisions - 1 { continue; }

                let index = (6 * (x * (nb_subdivisions - 1) + y)) as usize;
                indices[index] = (x + 1) * nb_subdivisions + y;
                indices[index + 1] = x * nb_subdivisions + y;
                indices[index + 2] = x * nb_subdivisions + y + 1;
                indices[index + 3] = (x + 1) * nb_subdivisions + y;
                indices[index + 4] = x * nb_subdivisions + y + 1;
                indices[index + 5] = (x + 1) * nb_subdivisions + y + 1;
            }
        }

        Mesh::from_data(indices, positions, engine)
    }
}