use crate::core::renderer::Renderer;
use crate::core::wgpu_context::WGPUContext;
use crate::materials::prepared_material::PreparedMaterial;

/// A material can be shared between meshes without sharing their object uniforms.
pub trait Material {
    fn prepare(&self, renderer: &mut Renderer, ctx: &WGPUContext) -> &PreparedMaterial;
}
