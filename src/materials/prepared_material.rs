use crate::core::renderer::Renderer;
use crate::core::wgpu_context::WGPUContext;
use crate::geometry::gpu_mesh_buffers::Vertex;
use crate::materials::utils::uniform_layout_entry;
use crate::texture::Texture;
use wgpu::{BindGroup, RenderPass, RenderPipeline};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PipelineConfig {
    pub shader_file: &'static str,
    pub polygon_mode: wgpu::PolygonMode,
    pub back_face_culling: bool,
}

pub struct PreparedMaterial {
    material_bind_group: BindGroup,
    pipeline: RenderPipeline,
}

impl PreparedMaterial {
    pub fn new(
        name: &str,
        config: PipelineConfig,
        uniforms: &wgpu::Buffer,
        textures: &[&Texture],
        renderer: &Renderer,
        ctx: &WGPUContext,
    ) -> Self {
        let shader_string = load_file::load_str!(config.shader_file);
        let shader = ctx
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(name),
                source: wgpu::ShaderSource::Wgsl(shader_string.into()),
            });
        let mut entries = vec![uniform_layout_entry(0)];
        entries.extend(
            textures.iter().enumerate().flat_map(|(i, texture)| {
                texture.create_bind_group_layout_entries(1 + 2 * i as u32)
            }),
        );
        let material_layout =
            ctx.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Material Bind Group Layout"),
                    entries: &entries,
                });
        let mut entries = vec![wgpu::BindGroupEntry {
            binding: 0,
            resource: uniforms.as_entire_binding(),
        }];
        entries.extend(
            textures
                .iter()
                .enumerate()
                .flat_map(|(i, texture)| texture.create_bind_group_entries(1 + 2 * i as u32)),
        );
        let material_bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(name),
            layout: &material_layout,
            entries: &entries,
        });
        let pipeline_layout = ctx
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(name),
                bind_group_layouts: &[
                    Some(&renderer.frame.layout),
                    Some(&material_layout),
                    Some(&ctx.object_bind_group_layout),
                ],
                immediate_size: 0,
            });
        let pipeline = ctx
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(&format!("{name} Render Pipeline")),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[Some(Vertex::desc())],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: ctx.config.format,
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
                    format: wgpu::TextureFormat::Depth24Plus,
                    depth_write_enabled: Some(true),
                    depth_compare: Some(wgpu::CompareFunction::LessEqual),
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState::default(),
                multiview_mask: None,
                cache: None,
            });

        Self {
            material_bind_group,
            pipeline,
        }
    }

    pub fn bind(&self, pass: &mut RenderPass<'_>) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(1, &self.material_bind_group, &[]);
    }
}
