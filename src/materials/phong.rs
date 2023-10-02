pub struct PhongUniforms {
    diffuse_color: [f32; 3],
    _padding1: u32,
    ambient_color: [f32; 3],
    _padding2: u32,
    specular_color: [f32; 3],
    _padding3: u32,
}