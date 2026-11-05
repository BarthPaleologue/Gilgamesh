use crate::core::render_target::DEPTH_FORMAT;
use crate::core::uniforms::uniform_layout_entry;
use crate::core::wgpu_context::WGPUContext;
use crate::geometry::gpu_mesh_buffers::Vertex;
use crate::texture;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct PipelineConfig {
    pub shader_source: &'static str,
    pub polygon_mode: wgpu::PolygonMode,
    pub back_face_culling: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct PipelineKey {
    pub(crate) config: PipelineConfig,
    pub(crate) texture_count: usize,
}

/// Pipeline state and material layout shared by compatible materials.
pub(crate) struct MaterialPipeline {
    pub(crate) pipeline: wgpu::RenderPipeline,
    pub(crate) material_layout: wgpu::BindGroupLayout,
}

impl MaterialPipeline {
    pub(crate) fn new(
        config: PipelineConfig,
        texture_count: usize,
        target_format: wgpu::TextureFormat,
        frame_layout: &wgpu::BindGroupLayout,
        object_layout: &wgpu::BindGroupLayout,
        ctx: &WGPUContext,
    ) -> Self {
        let shader = ctx
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Material Shader"),
                source: wgpu::ShaderSource::Wgsl(config.shader_source.into()),
            });
        let material_layout =
            ctx.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Material Bind Group Layout"),
                    entries: &material_bind_group_layout_entries(texture_count),
                });
        let pipeline_layout = ctx
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Material Pipeline Layout"),
                bind_group_layouts: &[
                    Some(frame_layout),
                    Some(&material_layout),
                    Some(object_layout),
                ],
                immediate_size: 0,
            });
        let pipeline = ctx
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Material Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[Some(Vertex::desc())],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: target_format,
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent::REPLACE,
                            alpha: wgpu::BlendComponent::REPLACE,
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    polygon_mode: config.polygon_mode,
                    cull_mode: if config.back_face_culling {
                        Some(wgpu::Face::Back)
                    } else {
                        None
                    },
                    ..Default::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: DEPTH_FORMAT,
                    depth_write_enabled: Some(true),
                    depth_compare: Some(wgpu::CompareFunction::GreaterEqual),
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState::default(),
                multiview_mask: None,
                cache: None,
            });

        Self {
            pipeline,
            material_layout,
        }
    }
}

fn material_bind_group_layout_entries(texture_count: usize) -> Vec<wgpu::BindGroupLayoutEntry> {
    let mut entries = vec![uniform_layout_entry(0)];
    entries.extend(
        (0..texture_count).flat_map(|i| texture::bind_group_layout_entries(1 + 2 * i as u32)),
    );
    entries
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn material_layouts_match_the_shader_binding_types_and_uniform_sizes() {
        for (source, texture_count, uniform_size) in [
            (
                include_str!("../shader/blinn_phong.wgsl"),
                4,
                std::mem::size_of::<crate::core::prepared_material::BlinnPhongUniforms>(),
            ),
            (
                include_str!("../shader/pbr.wgsl"),
                5,
                std::mem::size_of::<crate::core::prepared_material::PbrUniforms>(),
            ),
        ] {
            let module = naga::front::wgsl::parse_str(source).unwrap();
            let mut layouter = naga::proc::Layouter::default();
            layouter.update(module.to_ctx()).unwrap();
            let entries = material_bind_group_layout_entries(texture_count);
            let variables: Vec<_> = module
                .global_variables
                .iter()
                .filter(|(_, variable)| {
                    variable
                        .binding
                        .as_ref()
                        .is_some_and(|binding| binding.group == 1)
                })
                .collect();
            assert_eq!(entries.len(), variables.len());
            for entry in entries {
                let variable = variables
                    .iter()
                    .find(|(_, variable)| {
                        variable.binding.as_ref().unwrap().binding == entry.binding
                    })
                    .unwrap()
                    .1;
                match (&module.types[variable.ty].inner, entry.ty) {
                    (
                        naga::TypeInner::Struct { .. },
                        wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            ..
                        },
                    ) => {
                        assert_eq!(variable.space, naga::AddressSpace::Uniform);
                        assert_eq!(layouter[variable.ty].size as usize, uniform_size);
                    }
                    (
                        naga::TypeInner::Image {
                            dim: naga::ImageDimension::D2,
                            arrayed: false,
                            class:
                                naga::ImageClass::Sampled {
                                    kind: naga::ScalarKind::Float,
                                    multi: false,
                                },
                        },
                        wgpu::BindingType::Texture {
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                    )
                    | (
                        naga::TypeInner::Sampler { comparison: false },
                        wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    ) => {}
                    _ => panic!("incompatible material binding {}", entry.binding),
                }
            }
        }
    }
}
