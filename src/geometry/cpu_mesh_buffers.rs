use crate::geometry::cpu_vertex_buffers::CpuVertexBuffers;

/// CPU-side vertex attributes and triangle indices, before conversion for GPU upload.
#[derive(Debug, Default)]
pub struct CpuMeshBuffers {
    pub vertices: CpuVertexBuffers,
    pub indices: Vec<u32>,
}

impl CpuMeshBuffers {
    pub fn create_normals(&mut self) {
        let positions = &self.vertices.positions;
        let indices = &self.indices;
        let mut normals = vec![[0.0, 0.0, 0.0]; positions.len()];

        for i in 0..indices.len() / 3 {
            let i0 = indices[i * 3] as usize;
            let i1 = indices[i * 3 + 1] as usize;
            let i2 = indices[i * 3 + 2] as usize;

            let edge1 = [
                positions[i1][0] - positions[i0][0],
                positions[i1][1] - positions[i0][1],
                positions[i1][2] - positions[i0][2],
            ];
            let edge2 = [
                positions[i2][0] - positions[i0][0],
                positions[i2][1] - positions[i0][1],
                positions[i2][2] - positions[i0][2],
            ];
            let normal = [
                edge1[1] * edge2[2] - edge1[2] * edge2[1],
                edge1[2] * edge2[0] - edge1[0] * edge2[2],
                edge1[0] * edge2[1] - edge1[1] * edge2[0],
            ];

            normals[i0][0] += normal[0];
            normals[i0][1] += normal[1];
            normals[i0][2] += normal[2];

            normals[i1][0] += normal[0];
            normals[i1][1] += normal[1];
            normals[i1][2] += normal[2];

            normals[i2][0] += normal[0];
            normals[i2][1] += normal[1];
            normals[i2][2] += normal[2];
        }

        // Normalize normals
        for normal in normals.iter_mut() {
            let length =
                (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
            normal[0] /= length;
            normal[1] /= length;
            normal[2] /= length;
        }

        self.vertices.normals = normals;
    }

    pub fn create_tangents(&mut self) {
        let positions = &self.vertices.positions;
        let uvs = &self.vertices.uvs;
        let indices = &self.indices;
        let mut tangents = vec![[0.0, 0.0, 0.0]; positions.len()];

        for i in 0..indices.len() / 3 {
            let i0 = indices[i * 3] as usize;
            let i1 = indices[i * 3 + 1] as usize;
            let i2 = indices[i * 3 + 2] as usize;

            let edge1 = [
                positions[i1][0] - positions[i0][0],
                positions[i1][1] - positions[i0][1],
                positions[i1][2] - positions[i0][2],
            ];
            let edge2 = [
                positions[i2][0] - positions[i0][0],
                positions[i2][1] - positions[i0][1],
                positions[i2][2] - positions[i0][2],
            ];

            let delta_uv1 = [uvs[i1][0] - uvs[i0][0], uvs[i1][1] - uvs[i0][1]];
            let delta_uv2 = [uvs[i2][0] - uvs[i0][0], uvs[i2][1] - uvs[i0][1]];

            let f = 1.0 / (delta_uv1[0] * delta_uv2[1] - delta_uv2[0] * delta_uv1[1]);

            let tangent = [
                f * (delta_uv2[1] * edge1[0] - delta_uv1[1] * edge2[0]),
                f * (delta_uv2[1] * edge1[1] - delta_uv1[1] * edge2[1]),
                f * (delta_uv2[1] * edge1[2] - delta_uv1[1] * edge2[2]),
            ];

            tangents[i0][0] += tangent[0];
            tangents[i0][1] += tangent[1];
            tangents[i0][2] += tangent[2];

            tangents[i1][0] += tangent[0];
            tangents[i1][1] += tangent[1];
            tangents[i1][2] += tangent[2];

            tangents[i2][0] += tangent[0];
            tangents[i2][1] += tangent[1];
            tangents[i2][2] += tangent[2];
        }

        // Normalize tangents
        for tangent in tangents.iter_mut() {
            let length =
                (tangent[0] * tangent[0] + tangent[1] * tangent[1] + tangent[2] * tangent[2])
                    .sqrt();
            tangent[0] /= length;
            tangent[1] /= length;
            tangent[2] /= length;
        }

        self.vertices.tangents = tangents;
    }
}
