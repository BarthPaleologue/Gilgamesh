use crate::geometry::cpu_vertex_buffers::CpuVertexBuffers;
use glam::{Vec2, Vec3};

/// CPU-side vertex attributes and triangle indices, before conversion for GPU upload.
#[derive(Debug, Default)]
pub struct CpuMeshBuffers {
    pub vertices: CpuVertexBuffers,
    pub indices: Vec<u32>,
}

impl CpuMeshBuffers {
    pub fn create_normals(&mut self) {
        let positions = &self.vertices.positions;
        let mut normals = vec![Vec3::ZERO; positions.len()];

        for triangle in self.indices.as_chunks::<3>().0 {
            let [i0, i1, i2] = triangle.map(|index| index as usize);
            let [a, b, c] = [i0, i1, i2].map(|index| Vec3::from_array(positions[index]));
            let normal = (b - a).cross(c - a);
            for index in [i0, i1, i2] {
                normals[index] += normal;
            }
        }

        self.vertices.normals = normals
            .into_iter()
            .map(|normal| normal.normalize().to_array())
            .collect();
    }

    pub fn create_tangents(&mut self) {
        let positions = &self.vertices.positions;
        let uvs = &self.vertices.uvs;
        let mut tangents = vec![Vec3::ZERO; positions.len()];

        for triangle in self.indices.as_chunks::<3>().0 {
            let [i0, i1, i2] = triangle.map(|index| index as usize);
            let [a, b, c] = [i0, i1, i2].map(|index| Vec3::from_array(positions[index]));
            let [uv_a, uv_b, uv_c] = [i0, i1, i2].map(|index| Vec2::from_array(uvs[index]));
            let delta_uv1 = uv_b - uv_a;
            let delta_uv2 = uv_c - uv_a;
            let inverse_determinant = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv2.x * delta_uv1.y);
            let tangent = inverse_determinant * (delta_uv2.y * (b - a) - delta_uv1.y * (c - a));
            for index in [i0, i1, i2] {
                tangents[index] += tangent;
            }
        }

        self.vertices.tangents = tangents
            .into_iter()
            .map(|tangent| tangent.normalize().to_array())
            .collect();
    }
}
