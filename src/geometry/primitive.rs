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
        let colors = vec!([1.0, 1.0, 1.0]; (nb_subdivisions * nb_subdivisions) as usize);

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

    pub fn cube(name: &str, engine: &mut Engine) -> Mesh {
        let mut vertex_data = VertexData::default();
        vertex_data.positions = vec![
            [-1.0, -1.0, -1.0],
            [1.0, -1.0, -1.0],
            [-1.0, 1.0, -1.0],
            [1.0, 1.0, -1.0],
            [-1.0, -1.0, 1.0],
            [1.0, -1.0, 1.0],
            [-1.0, 1.0, 1.0],
            [1.0, 1.0, 1.0],
        ];
        vertex_data.colors = vec![
            [1.0, 1.0, 1.0],
            [1.0, 0.0, 1.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 1.0],
            [1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0],
        ];
        vertex_data.indices = vec![
            0, 2, 3,
            0, 3, 1,
            0, 1, 5,
            0, 5, 4,
            0, 4, 6,
            0, 6, 2,
            1, 3, 7,
            1, 7, 5,
            2, 6, 7,
            2, 7, 3,
            4, 5, 7,
            4, 7, 6,
        ];
        vertex_data.normals = vec![
            [-0.577350, -0.577350, -0.577350],
            [0.816497, -0.408248, -0.408248],
            [-0.408248, 0.816497, -0.408248],
            [0.408248, 0.408248, -0.816497],
            [-0.408248, -0.408248, 0.816497],
            [0.408248, -0.816497, 0.408248],
            [-0.816497, 0.408248, 0.408248],
            [0.577350, 0.577350, 0.577350],
        ];
        vertex_data.uvs = vec![
            [0.0, 1.0],
            [1.0, 1.0],
            [1.0, 0.0],
            [0.0, 0.0],
            [0.0, 0.0],
            [1.0, 0.0],
            [1.0, 1.0],
            [0.0, 1.0],
        ];

        Mesh::from_vertex_data(name, vertex_data, engine)
    }

    pub fn sphere(name: &str, resolution: u32, engine: &mut Engine) -> Mesh {
        let mut positions = Vec::new();
        let mut normals = Vec::new();
        let mut uvs = Vec::new();
        let mut indices = Vec::new();
        let mut colors = Vec::new();

        let pi = std::f32::consts::PI;

        let sector_step = 2.0 * pi / (resolution as f32 - 1.0);
        let stack_step = pi / (resolution as f32 - 1.0);

        for i in 0..resolution {
            let stack_angle = pi / 2.0 - i as f32 * stack_step;        // starting from pi/2 to -pi/2
            let xy = stack_angle.cos();             // r * cos(u)
            let z = stack_angle.sin();              // r * sin(u)

            // add (sectorCount+1) positions per stack
            // the first and last positions have same position and normal, but different tex coords
            for j in 0..resolution {
                let sector_angle = j as f32 * sector_step;           // starting from 0 to 2pi

                // vertex position
                let x = xy * sector_angle.cos();             // r * cos(u) * cos(v)
                let y = xy * sector_angle.sin();             // r * cos(u) * sin(v)
                positions.push([x, z, y]);

                // normalized vertex normal
                let nx = x;
                let ny = z;
                let nz = y;
                normals.push([nx, ny, nz]);

                // vertex tex coord between [0, 1]
                let s = j as f32 / (resolution as f32 - 1.0);
                let t = i as f32 / (resolution as f32 - 1.0);
                uvs.push([1.0 - s, t]);

                colors.push([1.0, 1.0, 1.0]);
            }
        }

        // indices
        //  k1--k1+1
        //  |  / |
        //  | /  |
        //  k2--k2+1
        for i in 0..resolution - 1 {
            let k1 = i * resolution;     // beginning of current stack
            let k2 = k1 + resolution;    // beginning of next stack

            for j in 0..resolution - 1 {
                // 2 triangles per sector excluding first and last stacks
                // k1 => k2 => k1+1
                indices.push(k1 + j);
                indices.push(k1 + j + 1);
                indices.push(k2 + j);

                // k1+1 => k2 => k2+1
                indices.push(k1 + j + 1);
                indices.push(k2 + j + 1);
                indices.push(k2 + j);
            }
        }

        let vertex_data = VertexData {
            positions,
            normals,
            uvs,
            indices,
            colors,
        };

        Mesh::from_vertex_data(name, vertex_data, engine)
    }
}