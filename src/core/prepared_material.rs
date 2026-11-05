use crate::core::material_pipeline::{MaterialPipeline, PipelineConfig};
use crate::core::raster_renderer::RasterRenderer;
use crate::core::wgpu_context::WGPUContext;
use crate::materials::material::Material;
use std::rc::Rc;
use wgpu::util::DeviceExt;
use wgpu::{BindGroup, RenderPass};

/// GPU bindings for one material, independent of object and frame data.
pub(crate) struct PreparedMaterial {
    material_bind_group: BindGroup,
    pipeline: Rc<MaterialPipeline>,
}

impl PreparedMaterial {
    pub(crate) fn new(
        material: &dyn Material,
        polygon_mode: wgpu::PolygonMode,
        renderer: &mut RasterRenderer,
        ctx: &WGPUContext,
    ) -> Self {
        let raster = material.raster();
        let config = PipelineConfig {
            shader_source: raster.shader_source,
            texture_count: raster.textures.len(),
            polygon_mode,
            back_face_culling: !raster.double_sided,
        };
        let pipeline = renderer.material_pipeline(config, ctx);
        let buffer = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(material.name()),
                contents: &raster.uniforms,
                usage: wgpu::BufferUsages::UNIFORM,
            });
        let mut entries = vec![wgpu::BindGroupEntry {
            binding: 0,
            resource: buffer.as_entire_binding(),
        }];
        entries.extend(raster.textures.iter().enumerate().flat_map(|(i, texture)| {
            texture
                .unwrap_or(&renderer.fallback_texture)
                .create_bind_group_entries(
                    1 + 2 * u32::try_from(i).expect("Material texture index fits in u32"),
                )
        }));
        let material_bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(material.name()),
            layout: &pipeline.material_layout,
            entries: &entries,
        });

        Self {
            material_bind_group,
            pipeline,
        }
    }

    pub(crate) fn bind(&self, pass: &mut RenderPass<'_>) {
        pass.set_pipeline(&self.pipeline.pipeline);
        pass.set_bind_group(1, &self.material_bind_group, &[]);
    }
}
