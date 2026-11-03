use hexasphere::shapes::IcoSphere;

use crate::geometry::cpu_mesh_buffers::CpuMeshBuffers;
use crate::geometry::cpu_vertex_buffers::CpuVertexBuffers;

/// Creates a new procedural 2D terrain.
/// It is made of a subdivided plane, with a given `size` and number of subdivisions (`nb_subdivisions`)
/// The `height_fn` takes x and z as parameters and is used to set the y coordinate of each vertex.
/// Returns CPU mesh buffers without allocating GPU resources.
pub fn terrain(
    size: f32,
    nb_subdivisions: u32,
    height_fn: &dyn Fn(f32, f32) -> f32,
) -> CpuMeshBuffers {
    let mut positions = vec![[0.0, 0.0, 0.0]; (nb_subdivisions * nb_subdivisions) as usize];
    let mut indices = vec![0; (6 * (nb_subdivisions - 1) * (nb_subdivisions - 1)) as usize];

    for x in 0..nb_subdivisions {
        let actual_x = (x as f32 / (nb_subdivisions as f32 - 1.0)) * size - size / 2.0;

        for y in 0..nb_subdivisions {
            let actual_y = (y as f32 / (nb_subdivisions as f32 - 1.0)) * size - size / 2.0;

            positions[(x * nb_subdivisions + y) as usize] =
                [actual_x, height_fn(actual_x, actual_y), actual_y];

            if x == nb_subdivisions - 1 || y == nb_subdivisions - 1 {
                continue;
            }

            let index = (6 * (x * (nb_subdivisions - 1) + y)) as usize;
            indices[index] = (x + 1) * nb_subdivisions + y;
            indices[index + 1] = x * nb_subdivisions + y;
            indices[index + 2] = x * nb_subdivisions + y + 1;
            indices[index + 3] = (x + 1) * nb_subdivisions + y;
            indices[index + 4] = x * nb_subdivisions + y + 1;
            indices[index + 5] = (x + 1) * nb_subdivisions + y + 1;
        }
    }

    let nb_vertices = positions.len();
    let mut cpu_buffers = CpuMeshBuffers {
        vertices: CpuVertexBuffers {
            positions,
            colors: vec![[0.6, 0.6, 0.6]; nb_vertices],
            normals: vec![[0.0, 0.0, 0.0]; nb_vertices],
            tangents: vec![[0.0, 0.0, 0.0]; nb_vertices],
            uvs: vec![[0.0, 0.0]; nb_vertices],
        },
        indices,
    };
    cpu_buffers.create_normals();
    cpu_buffers.create_tangents();

    cpu_buffers
}

/// Creates a new procedural 3D sphere.
/// It is made of a subdivided icosahedron, with a given `diameter` and number of subdivisions (`nb_subdivisions`)
/// The `height_fn` takes x, y and z as parameters and is used to set the height of each vertex above the surface of the sphere.
pub fn sphere(
    diameter: f32,
    nb_subdivisions: u32,
    height_fn: &dyn Fn(f32, f32, f32) -> f32,
) -> CpuMeshBuffers {
    let sphere = IcoSphere::new(nb_subdivisions as usize, |_| ());
    let vertices_raw = sphere.raw_points();
    let mut vertices: Vec<[f32; 3]> = Vec::with_capacity(vertices_raw.len());

    for point in vertices_raw {
        let vertex = [point[0], point[1], point[2]];
        let unit_vertex = [vertex[0], vertex[1], vertex[2]];

        let height = height_fn(vertex[0], vertex[1], vertex[2]);
        let original_vertex = [
            vertex[0] * diameter / 2.0,
            vertex[1] * diameter / 2.0,
            vertex[2] * diameter / 2.0,
        ];

        let moved_vertex = [
            original_vertex[0] + unit_vertex[0] * height,
            original_vertex[1] + unit_vertex[1] * height,
            original_vertex[2] + unit_vertex[2] * height,
        ];

        vertices.push([moved_vertex[0], moved_vertex[1], moved_vertex[2]]);
    }

    let indices = sphere.get_all_indices();

    let nb_vertices = vertices.len();
    let mut cpu_buffers = CpuMeshBuffers {
        vertices: CpuVertexBuffers {
            positions: vertices,
            colors: vec![[0.6, 0.6, 0.6]; nb_vertices],
            normals: vec![[0.0, 0.0, 0.0]; nb_vertices],
            tangents: vec![[0.0, 0.0, 0.0]; nb_vertices],
            uvs: vec![[0.0, 0.0]; nb_vertices],
        },
        indices,
    };
    cpu_buffers.create_normals();
    cpu_buffers.create_tangents();

    cpu_buffers
}
