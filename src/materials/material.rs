use crate::core::renderer::Renderer;
use crate::core::wgpu_context::WGPUContext;
use wgpu::RenderPass;

pub trait Material {
    fn bind(
        &mut self,
        render_pass: &mut RenderPass<'_>,
        renderer: &mut Renderer,
        ctx: &WGPUContext,
    );
}
