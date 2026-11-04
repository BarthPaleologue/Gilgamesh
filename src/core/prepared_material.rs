use crate::core::material_pipeline::{MaterialPipeline, PipelineConfig};
use crate::core::raster_renderer::RasterRenderer;
use crate::core::wgpu_context::WGPUContext;
use crate::materials::material::Material;
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

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct BlinnPhongUniforms {
    diffuse_color: [f32; 3],
    has_diffuse_texture: u32,
    ambient_color: [f32; 3],
    has_ambient_texture: u32,
    specular_color: [f32; 3],
    has_specular_texture: u32,
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
        material: &Material,
        polygon_mode: wgpu::PolygonMode,
        renderer: &mut RasterRenderer,
        ctx: &WGPUContext,
    ) -> Self {
        match material {
            Material::Pbr(material) => {
                let uniforms = PbrUniforms::from(material);
                Self::from_uniforms(
                    &material.name,
                    PipelineConfig {
                        shader_source: include_str!("../shader/pbr.wgsl"),
                        polygon_mode,
                        back_face_culling: !material.double_sided,
                    },
                    bytemuck::bytes_of(&uniforms),
                    &[
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
                    ],
                    renderer,
                    ctx,
                )
            }
            Material::BlinnPhong(material) => {
                let uniforms = BlinnPhongUniforms::from(material);
                Self::from_uniforms(
                    &material.name,
                    PipelineConfig {
                        shader_source: include_str!("../shader/blinn_phong.wgsl"),
                        polygon_mode,
                        back_face_culling: !material.double_sided,
                    },
                    bytemuck::bytes_of(&uniforms),
                    &[
                        material
                            .diffuse_texture
                            .as_ref()
                            .map(|texture| texture as &dyn SampledTexture),
                        material
                            .ambient_texture
                            .as_ref()
                            .map(|texture| texture as &dyn SampledTexture),
                        material
                            .specular_texture
                            .as_deref()
                            .map(|texture| texture as &dyn SampledTexture),
                        material
                            .normal_map
                            .as_deref()
                            .map(|texture| texture as &dyn SampledTexture),
                    ],
                    renderer,
                    ctx,
                )
            }
        }
    }

    fn from_uniforms(
        name: &str,
        config: PipelineConfig,
        uniforms: &[u8],
        textures: &[Option<&dyn SampledTexture>],
        renderer: &mut RasterRenderer,
        ctx: &WGPUContext,
    ) -> Self {
        let pipeline = renderer.material_pipeline(config, textures.len(), ctx);
        let buffer = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(name),
                contents: uniforms,
                usage: wgpu::BufferUsages::UNIFORM,
            });
        let mut entries = vec![wgpu::BindGroupEntry {
            binding: 0,
            resource: buffer.as_entire_binding(),
        }];
        entries.extend(textures.iter().enumerate().flat_map(|(i, texture)| {
            texture
                .unwrap_or(&renderer.fallback_texture)
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

impl From<&crate::materials::pbr::PbrMaterial> for PbrUniforms {
    fn from(material: &crate::materials::pbr::PbrMaterial) -> Self {
        PbrUniforms {
            albedo_color: material.albedo_color,
            has_albedo_texture: u32::from(material.albedo_texture.is_some()),
            emissive_color: material.emissive_color,
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

impl From<&crate::materials::blinn_phong::BlinnPhongMaterial> for BlinnPhongUniforms {
    fn from(material: &crate::materials::blinn_phong::BlinnPhongMaterial) -> Self {
        BlinnPhongUniforms {
            diffuse_color: material.diffuse_color,
            has_diffuse_texture: u32::from(material.diffuse_texture.is_some()),
            ambient_color: material.ambient_color,
            has_ambient_texture: u32::from(material.ambient_texture.is_some()),
            specular_color: material.specular_color,
            has_specular_texture: u32::from(material.specular_texture.is_some()),
            has_normal_map: u32::from(material.normal_map.is_some()),
            _padding: [0; 3],
        }
    }
}

#[cfg(test)]
mod pbr_tests {
    use super::PbrUniforms;
    use crate::materials::pbr::PbrMaterial;

    #[test]
    fn constant_colors_can_be_configured_without_texture_resources() {
        let mut material = PbrMaterial::new("Constant");
        assert!(material.albedo_texture.is_none());
        assert!(material.emissive_texture.is_none());
        assert!(material.normal_map.is_none());
        assert!(material.roughness_map.is_none());
        material.set_albedo_color(0.2, 0.3, 0.4);
        material.set_emissive_color(0.1, 0.0, 0.2);

        let uniforms = PbrUniforms::from(&material);
        assert_eq!(uniforms.albedo_color, [0.2, 0.3, 0.4]);
        assert_eq!(uniforms.emissive_color, [0.1, 0.0, 0.2]);
        assert_eq!(uniforms.metallic, 0.7);
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

#[cfg(test)]
mod blinn_phong_tests {
    use super::BlinnPhongUniforms;
    use crate::materials::blinn_phong::BlinnPhongMaterial;

    #[test]
    fn constant_colors_can_be_configured_without_texture_resources() {
        let mut material = BlinnPhongMaterial::new("Constant");
        assert!(material.diffuse_texture.is_none());
        assert!(material.ambient_texture.is_none());
        assert!(material.specular_texture.is_none());
        assert!(material.normal_map.is_none());
        material.set_diffuse_color(0.2, 0.3, 0.4);
        material.set_ambient_color(0.1, 0.0, 0.2);
        material.set_specular_color(0.8, 0.7, 0.6);

        let uniforms = BlinnPhongUniforms::from(&material);
        assert_eq!(uniforms.diffuse_color, [0.2, 0.3, 0.4]);
        assert_eq!(uniforms.ambient_color, [0.1, 0.0, 0.2]);
        assert_eq!(uniforms.specular_color, [0.8, 0.7, 0.6]);
        assert_eq!(
            [
                uniforms.has_diffuse_texture,
                uniforms.has_ambient_texture,
                uniforms.has_specular_texture,
                uniforms.has_normal_map,
            ],
            [0; 4],
        );
    }
}
