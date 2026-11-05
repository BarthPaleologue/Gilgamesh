use crate::core::material_pipeline::{MaterialPipeline, PipelineConfig};
use crate::core::raster_renderer::RasterRenderer;
use crate::core::wgpu_context::WGPUContext;
use crate::materials::pbr::PbrMaterial;
use crate::texture::SampledTexture;
use std::rc::Rc;
use wgpu::util::DeviceExt;
use wgpu::{BindGroup, RenderPass};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct PbrUniforms {
    albedo_color: [f32; 3],
    has_albedo_texture: u32,

    emissive_color: [f32; 3],
    has_emissive_texture: u32,

    metallic: f32,
    has_metallic_texture: u32,
    roughness: f32,
    has_roughness_texture: u32,

    has_normal_map: u32,
    _padding: [u32; 3],
}

/// GPU bindings for one material, independent of object and frame data.
pub(crate) struct PreparedMaterial {
    material_bind_group: BindGroup,
    pipeline: Rc<MaterialPipeline>,
}

impl PreparedMaterial {
    pub(crate) fn new(
        material: &PbrMaterial,
        polygon_mode: wgpu::PolygonMode,
        renderer: &mut RasterRenderer,
        ctx: &WGPUContext,
    ) -> Self {
        let uniforms = PbrUniforms::from(material);
        let config = PipelineConfig {
            polygon_mode,
            back_face_culling: !material.double_sided,
        };
        let textures: [Option<&dyn SampledTexture>; 5] = [
            material
                .albedo_texture
                .as_ref()
                .map(|texture| texture as &dyn SampledTexture),
            material
                .emissive_texture
                .as_ref()
                .map(|texture| texture as &dyn SampledTexture),
            material
                .normal_map
                .as_deref()
                .map(|texture| texture as &dyn SampledTexture),
            None,
            material
                .roughness_map
                .as_deref()
                .map(|texture| texture as &dyn SampledTexture),
        ];
        let pipeline = renderer.material_pipeline(config, ctx);
        let buffer = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&material.name),
                contents: bytemuck::bytes_of(&uniforms),
                usage: wgpu::BufferUsages::UNIFORM,
            });
        let mut entries = vec![wgpu::BindGroupEntry {
            binding: 0,
            resource: buffer.as_entire_binding(),
        }];
        entries.extend(textures.iter().enumerate().flat_map(|(i, texture)| {
            texture
                .unwrap_or(&renderer.fallback_texture)
                .create_bind_group_entries(
                    1 + 2 * u32::try_from(i).expect("Material texture index fits in u32"),
                )
        }));
        let material_bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&material.name),
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

impl From<&crate::materials::PbrMaterial> for PbrUniforms {
    fn from(material: &crate::materials::PbrMaterial) -> Self {
        PbrUniforms {
            albedo_color: material.albedo_color,
            has_albedo_texture: u32::from(material.albedo_texture.is_some()),
            emissive_color: material
                .emissive_color
                .map(|c| c * material.emissive_luminance.value()),
            has_emissive_texture: u32::from(material.emissive_texture.is_some()),
            metallic: material.metallic,
            has_metallic_texture: 0,
            roughness: material.roughness,
            has_roughness_texture: u32::from(material.roughness_map.is_some()),
            has_normal_map: u32::from(material.normal_map.is_some()),
            _padding: [0; 3],
        }
    }
}

#[cfg(test)]
mod pbr_tests {
    use super::PbrUniforms;
    use crate::materials::PbrMaterial;
    use crate::photometry::Nits;

    #[test]
    fn constant_colors_can_be_configured_without_texture_resources() {
        let mut material = PbrMaterial::new("Constant");
        assert!(material.albedo_texture.is_none());
        assert!(material.emissive_texture.is_none());
        assert_eq!(PbrUniforms::from(&material).emissive_color, [0.0; 3]);
        assert!(material.normal_map.is_none());
        assert!(material.roughness_map.is_none());
        material.set_albedo_color(0.2, 0.3, 0.4);
        material.set_emissive_luminance(Nits::new(2.0));
        material.set_emissive_color(0.1, 0.0, 0.2);

        let uniforms = PbrUniforms::from(&material);
        assert_eq!(uniforms.albedo_color, [0.2, 0.3, 0.4]);
        assert_eq!(uniforms.emissive_color, [0.2, 0.0, 0.4]);
        assert_eq!(uniforms.metallic, 0.0);
        assert_eq!(uniforms.roughness, 0.4);
        assert_eq!(
            [
                uniforms.has_albedo_texture,
                uniforms.has_emissive_texture,
                uniforms.has_normal_map,
                uniforms.has_metallic_texture,
                uniforms.has_roughness_texture,
            ],
            [0; 5],
        );
    }

    #[test]
    fn gpu_uniforms_match_shader_layout() {
        assert_eq!(std::mem::size_of::<PbrUniforms>(), 64);
        assert_eq!(std::mem::offset_of!(PbrUniforms, emissive_color), 16);
        assert_eq!(std::mem::offset_of!(PbrUniforms, metallic), 32);
        assert_eq!(std::mem::offset_of!(PbrUniforms, roughness), 40);
    }
}
