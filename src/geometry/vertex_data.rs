pub struct VertexData {
    pub positions: Vec<[f32; 3]>,
    pub colors: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub uvs: Vec<[f32; 2]>,
}

impl Default for VertexData {
    fn default() -> Self {
        VertexData {
            positions: Vec::new(),
            colors: Vec::new(),
            normals: Vec::new(),
            indices: Vec::new(),
            uvs: Vec::new(),
        }
    }
}

impl VertexData {
    pub fn create_normals(&mut self) {
        let positions = &self.positions;
        let indices = &self.indices;
        let mut normals = vec![[0.0, 0.0, 0.0]; positions.len()];

        for i in 0..indices.len() / 3 {
            let i0 = indices[i * 3 + 0] as usize;
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
            let length = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
            normal[0] /= length;
            normal[1] /= length;
            normal[2] /= length;
        }

        self.normals = normals;
    }
}