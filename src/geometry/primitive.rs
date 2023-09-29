use crate::core::engine::Engine;
use crate::geometry::mesh::Mesh;
use crate::geometry::vertex_data::VertexData;

pub struct PrimitiveMesh {}

impl PrimitiveMesh {
    pub fn plane(name: &str, nb_subdivisions: u32, size: f32, engine: &mut Engine) -> Mesh {
        let mut positions = vec!([0.0, 0.0, 0.0]; (nb_subdivisions * nb_subdivisions) as usize);
        let mut normals = vec!([0.0, 0.0, 0.0]; (nb_subdivisions * nb_subdivisions) as usize);
        let mut indices = vec!(0; (6 * (nb_subdivisions - 1) * (nb_subdivisions - 1)) as usize);
        let mut uvs = vec!([0.0, 0.0]; (nb_subdivisions * nb_subdivisions) as usize);
        let mut colors = vec!([1.0, 1.0, 1.0]; (nb_subdivisions * nb_subdivisions) as usize);

        for x in 0..nb_subdivisions {
            let actual_x = (x as f32 / (nb_subdivisions as f32 - 1.0)) * size - size / 2.0;
            let uv_x = x as f32 / (nb_subdivisions as f32 - 1.0);

            for y in 0..nb_subdivisions {
                let actual_y = (y as f32 / (nb_subdivisions as f32 - 1.0)) * size - size / 2.0;
                let uv_y = y as f32 / (nb_subdivisions as f32 - 1.0);

                uvs[(x * nb_subdivisions + y) as usize] = [uv_x, uv_y];

                positions[(x * nb_subdivisions + y) as usize] = [actual_x, 0.0, actual_y];
                normals[(x * nb_subdivisions + y) as usize] = [0.0, 1.0, 0.0];

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

        let vertex_data = VertexData {
            positions,
            colors,
            normals,
            indices,
            uvs,
        };

        Mesh::from_vertex_data(name, vertex_data, engine)
    }
}