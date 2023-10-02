use std::cell::Ref;
use std::num::NonZeroU32;
use std::ops::Deref;
use bytemuck::{cast_slice};
use load_file::load_str;
use wgpu::{BindGroup, Buffer, PipelineLayout, RenderPass, RenderPipeline, ShaderModule};
use crate::camera::camera::Camera;

use crate::geometry::mesh::Vertex;
use crate::core::wgpu_context::WGPUContext;
use crate::lights::directional_light::{DirectionalLight};
use crate::lights::point_light::{PointLight};
use crate::materials::utils::{create_buffer};
use crate::settings::MAX_POINT_LIGHTS;
use crate::transform::{Transform, TransformUniforms};

pub struct MaterialPipeline {
    pub shader_module: ShaderModule,

    pub transform_uniforms: TransformUniforms,
    pub transform_uniforms_buffer: Buffer,
    pub transform_bind_group: BindGroup,

    pub uniform_bind_group: BindGroup,

    pub pipeline_layout: PipelineLayout,
    pub pipeline: RenderPipeline,
}

impl MaterialPipeline {
    pub fn new(shader_file: &str, uniforms: &[&Buffer], wgpu_context: &mut WGPUContext) -> MaterialPipeline {
        // load shader from file at runtime
        let shader_string = load_str!(shader_file);
        let shader = wgpu_context.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_string.into()),
        });

        let transform_uniforms = TransformUniforms::default();
        let transform_uniforms_buffer = create_buffer::<TransformUniforms>("Transform Buffer", wgpu_context);

        let transform_bind_group_layout = wgpu_context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Transform Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let transform_bind_group = wgpu_context.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &transform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: transform_uniforms_buffer.as_entire_binding(),
            }],
            label: Some("Transform Bind Group"),
        });

        let entries: Vec<wgpu::BindGroupLayoutEntry> = uniforms.iter().enumerate().map(|(i, binding_resource)| {
            wgpu::BindGroupLayoutEntry {
                binding: i as u32,
                visibility: wgpu::ShaderStages::all(),
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None, //NonZeroU32::new(MAX_POINT_LIGHTS as u32),
            }
        }).collect();

        let uniform_bind_group_layout = wgpu_context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &entries,
            label: Some("Uniform Bind Group Layout"),
        });

        let entries: Vec<wgpu::BindGroupEntry> = uniforms.iter().enumerate().map(|(i, binding_resource)| {
            wgpu::BindGroupEntry {
                binding: i as u32,
                resource: binding_resource.as_entire_binding(),
            }
        }).collect();

        let uniform_bind_group = wgpu_context.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &entries,
            label: Some("Uniform Bind Group"),
        });

        let pipeline_layout = wgpu_context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                &transform_bind_group_layout,
                &uniform_bind_group_layout
            ],
            push_constant_ranges: &[],
        });

        let pipeline = wgpu_context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu_context.config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                polygon_mode: wgpu::PolygonMode::Fill,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth24Plus,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        MaterialPipeline {
            shader_module: shader,

            transform_uniforms,
            transform_uniforms_buffer,
            transform_bind_group,

            uniform_bind_group,

            pipeline_layout,
            pipeline,
        }
    }
    pub fn new_default(uniforms: &[&Buffer], wgpu_context: &mut WGPUContext) -> MaterialPipeline {
        MaterialPipeline::new("../shader/default.wgsl", uniforms, wgpu_context)
    }

    pub fn bind<'a, 'b>(&'a mut self, render_pass: &'b mut RenderPass<'a>, transform: Ref<Transform>, active_camera: &Camera, point_lights: &[PointLight], directional_light: &DirectionalLight, wgpu_context: &mut WGPUContext) {
        self.transform_uniforms.update(transform.deref());
        wgpu_context.queue.write_buffer(&self.transform_uniforms_buffer, 0, cast_slice(&[self.transform_uniforms]));

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.transform_bind_group, &[]);
        render_pass.set_bind_group(1, &self.uniform_bind_group, &[]);
    }
}