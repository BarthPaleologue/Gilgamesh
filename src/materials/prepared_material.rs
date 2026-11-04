use crate::core::renderer::Renderer;
use crate::core::wgpu_context::WGPUContext;
use crate::materials::material_pipeline::{MaterialPipeline, PipelineConfig};
use crate::texture::SampledTexture;
use std::rc::Rc;
use wgpu::{BindGroup, RenderPass};

/// GPU bindings for one material, independent of object and frame data.
pub struct PreparedMaterial {
    material_bind_group: BindGroup,
    pipeline: Rc<MaterialPipeline>,
}

impl PreparedMaterial {
    pub fn new(
        name: &str,
        config: PipelineConfig,
        uniforms: &wgpu::Buffer,
        textures: &[Option<&dyn SampledTexture>],
        renderer: &mut Renderer,
        ctx: &WGPUContext,
    ) -> Self {
        let pipeline = renderer.material_pipeline(config, textures.len(), ctx);
        let mut entries = vec![wgpu::BindGroupEntry {
            binding: 0,
            resource: uniforms.as_entire_binding(),
        }];
        entries.extend(textures.iter().enumerate().flat_map(|(i, texture)| {
            texture
                .unwrap_or(ctx.fallback_texture())
                .create_bind_group_entries(1 + 2 * i as u32)
        }));
        let material_bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(name),
            layout: &pipeline.material_layout,
            entries: &entries,
        });

        Self {
            material_bind_group,
            pipeline,
        }
    }

    pub fn bind(&self, pass: &mut RenderPass<'_>) {
        pass.set_pipeline(&self.pipeline.pipeline);
        pass.set_bind_group(1, &self.material_bind_group, &[]);
    }
}
