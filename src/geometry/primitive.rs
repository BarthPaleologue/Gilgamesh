use crate::geometry::cpu_mesh_buffers::CpuMeshBuffers;
use crate::geometry::cpu_vertex_buffers::CpuVertexBuffers;
use glam::Vec3;

pub fn plane(nb_subdivisions: u32, size: f32) -> CpuMeshBuffers {
    let mut positions = vec![[0.0, 0.0, 0.0]; (nb_subdivisions * nb_subdivisions) as usize];
    let mut normals = vec![[0.0, 0.0, 0.0]; (nb_subdivisions * nb_subdivisions) as usize];
    let mut tangents = vec![[0.0, 0.0, 0.0]; (nb_subdivisions * nb_subdivisions) as usize];
    let mut indices = vec![0; (6 * (nb_subdivisions - 1) * (nb_subdivisions - 1)) as usize];
    let mut uvs = vec![[0.0, 0.0]; (nb_subdivisions * nb_subdivisions) as usize];
    let colors = vec![[1.0, 1.0, 1.0]; (nb_subdivisions * nb_subdivisions) as usize];

    for x in 0..nb_subdivisions {
        let actual_x = (x as f32 / (nb_subdivisions as f32 - 1.0)) * size - size / 2.0;
        let uv_x = x as f32 / (nb_subdivisions as f32 - 1.0);

        for y in 0..nb_subdivisions {
            let actual_y = (y as f32 / (nb_subdivisions as f32 - 1.0)) * size - size / 2.0;
            let uv_y = y as f32 / (nb_subdivisions as f32 - 1.0);

            uvs[(x * nb_subdivisions + y) as usize] = [uv_x, uv_y];

            positions[(x * nb_subdivisions + y) as usize] = [actual_x, 0.0, actual_y];
            normals[(x * nb_subdivisions + y) as usize] = [0.0, 1.0, 0.0];
            tangents[(x * nb_subdivisions + y) as usize] = [1.0, 0.0, 0.0];

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

    CpuMeshBuffers {
        vertices: CpuVertexBuffers {
            positions,
            colors,
            normals,
            tangents,
            uvs,
        },
        indices,
    }
}

pub fn cube() -> CpuMeshBuffers {
    let corner_positions = [
        [-0.5, -0.5, -0.5],
        [0.5, -0.5, -0.5],
        [0.5, 0.5, -0.5],
        [-0.5, 0.5, -0.5],
        [-0.5, -0.5, 0.5],
        [0.5, -0.5, 0.5],
        [0.5, 0.5, 0.5],
        [-0.5, 0.5, 0.5],
    ];
    let corner_colors = [
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
        [1.0, 1.0, 0.0],
        [1.0, 0.0, 1.0],
        [0.0, 1.0, 1.0],
        [1.0, 1.0, 1.0],
        [0.0, 0.0, 0.0],
    ];
    let faces: [([usize; 4], [f32; 3]); 6] = [
        ([0, 1, 2, 3], [0.0, 0.0, -1.0]), // front
        ([1, 5, 6, 2], [1.0, 0.0, 0.0]),  // right
        ([5, 4, 7, 6], [0.0, 0.0, 1.0]),  // back
        ([4, 0, 3, 7], [-1.0, 0.0, 0.0]), // left
        ([3, 2, 6, 7], [0.0, 1.0, 0.0]),  // top
        ([4, 5, 1, 0], [0.0, -1.0, 0.0]), // bottom
    ];
    let face_uvs = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];

    let mut positions = Vec::with_capacity(faces.len() * 4);
    let mut normals = Vec::with_capacity(faces.len() * 4);
    let mut uvs = Vec::with_capacity(faces.len() * 4);
    let mut colors = Vec::with_capacity(faces.len() * 4);
    let mut indices = Vec::with_capacity(faces.len() * 6);

    for (corners, normal) in faces {
        let base = positions.len() as u32;
        for (corner, uv) in corners.into_iter().zip(face_uvs) {
            positions.push(corner_positions[corner]);
            normals.push(normal);
            uvs.push(uv);
            colors.push(corner_colors[corner]);
        }
        indices.extend_from_slice(&[base, base + 2, base + 1, base, base + 3, base + 2]);
    }

    let nb_vertices = positions.len();
    let mut cpu_buffers = CpuMeshBuffers {
        vertices: CpuVertexBuffers {
            positions,
            colors,
            normals,
            tangents: vec![[0.0, 0.0, 0.0]; nb_vertices],
            uvs,
        },
        indices,
    };

    cpu_buffers.create_tangents();

    cpu_buffers
}

pub fn sphere(resolution: u32) -> CpuMeshBuffers {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut tangents = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();
    let mut colors = Vec::new();

    let mut index = 0;
    for i in 0..resolution {
        let v = i as f32 / (resolution as f32 - 1.0);
        let theta = v * std::f32::consts::PI;
        let sin_theta = theta.sin();
        let cos_theta = theta.cos();

        for j in 0..resolution {
            let u = j as f32 / (resolution as f32 - 1.0);
            let phi = u * 2.0 * std::f32::consts::PI;
            let sin_phi = phi.sin();
            let cos_phi = phi.cos();

            let x = cos_phi * sin_theta;
            let y = cos_theta;
            let z = sin_phi * sin_theta;

            let tx = -sin_phi * sin_theta;
            let tz = cos_phi * sin_theta;

            positions.push([x, y, z]);
            normals.push([x, y, z]);
            tangents.push([tx, 0.0, tz]);
            uvs.push([1.0 - u, v]);
            colors.push([1.0, 1.0, 1.0]);

            if i < resolution - 1 && j < resolution - 1 {
                indices.push(index);
                indices.push(index + resolution + 1);
                indices.push(index + resolution);
                indices.push(index);
                indices.push(index + 1);
                indices.push(index + resolution + 1);
            }

            index += 1;
        }
    }

    CpuMeshBuffers {
        vertices: CpuVertexBuffers {
            positions,
            normals,
            tangents,
            uvs,
            colors,
        },
        indices,
    }
}

pub fn ico_sphere(nb_subdivisions: u32) -> CpuMeshBuffers {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut tangents = Vec::new();
    let mut uvs = Vec::new();
    let mut colors = Vec::new();

    let t = (1.0 + 5.0_f32.sqrt()) / 2.0;

    let mut base_positions = vec![
        Vec3::new(-1.0, t, 0.0).normalize(),
        Vec3::new(1.0, t, 0.0).normalize(),
        Vec3::new(-1.0, -t, 0.0).normalize(),
        Vec3::new(1.0, -t, 0.0).normalize(),
        Vec3::new(0.0, -1.0, t).normalize(),
        Vec3::new(0.0, 1.0, t).normalize(),
        Vec3::new(0.0, -1.0, -t).normalize(),
        Vec3::new(0.0, 1.0, -t).normalize(),
        Vec3::new(t, 0.0, -1.0).normalize(),
        Vec3::new(t, 0.0, 1.0).normalize(),
        Vec3::new(-t, 0.0, -1.0).normalize(),
        Vec3::new(-t, 0.0, 1.0).normalize(),
    ];

    let mut base_indices = vec![
        0, 11, 5, 0, 5, 1, 0, 1, 7, 0, 7, 10, 0, 10, 11, 1, 5, 9, 5, 11, 4, 11, 10, 2, 10, 7, 6, 7,
        1, 8, 3, 9, 4, 3, 4, 2, 3, 2, 6, 3, 6, 8, 3, 8, 9, 4, 9, 5, 2, 4, 11, 6, 2, 10, 8, 6, 7, 9,
        8, 1,
    ];

    for _ in 0..nb_subdivisions {
        let mut new_indices = Vec::new();
        for i in 0..base_indices.len() / 3 {
            let i0 = base_indices[i * 3];
            let i1 = base_indices[i * 3 + 1];
            let i2 = base_indices[i * 3 + 2];

            let v0 = base_positions[i0 as usize];
            let v1 = base_positions[i1 as usize];
            let v2 = base_positions[i2 as usize];

            let v01 = (v0 + v1).normalize();
            let v12 = (v1 + v2).normalize();
            let v20 = (v2 + v0).normalize();

            let i01 = base_positions.len() as u32;
            let i12 = i01 + 1;
            let i20 = i12 + 1;

            base_positions.push(v01);
            base_positions.push(v12);
            base_positions.push(v20);

            new_indices.push(i0);
            new_indices.push(i01);
            new_indices.push(i20);

            new_indices.push(i1);
            new_indices.push(i12);
            new_indices.push(i01);

            new_indices.push(i2);
            new_indices.push(i20);
            new_indices.push(i12);

            new_indices.push(i01);
            new_indices.push(i12);
            new_indices.push(i20);
        }
        base_indices = new_indices;
    }

    for v in base_positions {
        positions.push(v.into());
        normals.push(v.into());

        let theta = v.z.atan2(v.x);
        let phi = v.y.acos();
        let tx = -phi.sin() * theta.sin();
        let tz = phi.cos() * theta.sin();
        tangents.push([tx, 0.0, tz]);

        let u = theta / (2.0 * std::f32::consts::PI) + 0.5;
        let v = phi / std::f32::consts::PI;
        uvs.push([1.0 - u, v]);

        colors.push([1.0, 1.0, 1.0]);
    }

    CpuMeshBuffers {
        vertices: CpuVertexBuffers {
            positions,
            normals,
            tangents,
            uvs,
            colors,
        },
        indices: base_indices,
    }
}
