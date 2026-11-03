/// CPU-side arrays of per-vertex attributes, before conversion for GPU upload.
#[derive(Debug, Default)]
pub struct CpuVertexBuffers {
    pub positions: Vec<[f32; 3]>,
    pub colors: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub tangents: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
}
