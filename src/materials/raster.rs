use crate::texture::SampledTexture;

/// Raster shading over the engine's mesh vertex stage and frame/object bindings.
///
/// `shader_source` defines `fs_main(in: VertexOutput) -> @location(0) vec4<f32>`.
/// It can use `camera`, `directional_lights`, `point_lights`, `light_counts` and
/// `transform` from the engine's WGSL prelude. Output is scene-linear luminance.
/// `VertexOutput` carries `vPositionW`, `vColor`, local-space `vNormal` and
/// `vTangent`, and `vUV`. The shared vertex stage applies the camera and transform.
/// Group 1 binding 0 is the material uniform; texture/sampler pairs follow at
/// bindings 1/2, 3/4, etc. Texture slots containing `None` use the fallback texture.
/// RGB is linear; uniform field layout must match WGSL, including internal padding.
#[must_use]
pub struct RasterMaterial<'a> {
    pub shader_source: &'static str,
    pub textures: Vec<Option<&'a dyn SampledTexture>>,
    pub double_sided: bool,
    pub(crate) uniforms: Vec<u8>,
}

impl RasterMaterial<'_> {
    /// Copies a GPU-compatible uniform value, padding the buffer to 16 bytes for WebGL.
    pub fn new<T: bytemuck::Pod>(shader_source: &'static str, uniforms: T) -> Self {
        let mut bytes = bytemuck::bytes_of(&uniforms).to_vec();
        bytes.resize(bytes.len().max(1).next_multiple_of(16), 0);
        Self {
            shader_source,
            textures: Vec::new(),
            double_sided: false,
            uniforms: bytes,
        }
    }
}
