use bytemuck::{cast_slice};
use cgmath::{Matrix4, SquareMatrix};
use wgpu::{BindGroup, BindGroupLayout, Buffer, PipelineLayout, RenderPass, RenderPipeline, ShaderModule};
use wgpu::util::{DeviceExt};
use crate::camera::camera::Camera;
use crate::camera::uniforms::CameraUniforms;

use crate::geometry::mesh::Vertex;
use crate::core::wgpu_context::WGPUContext;

pub struct Material {
    pub shader_module: ShaderModule,

    pub vertex_uniform_buffer: Buffer,

    pub camera_uniforms: CameraUniforms,
    pub camera_uniforms_buffer: Buffer,
    pub camera_bind_group: BindGroup,

    pub uniform_bind_group_layout: BindGroupLayout,
    pub uniform_bind_group: BindGroup,

    pub pipeline_layout: PipelineLayout,
    pub pipeline: RenderPipeline,
}

impl Material {
    pub fn new(wgpu_context: &mut WGPUContext) -> Material {
        let shader = wgpu_context.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader/default.wgsl").into()),
        });

        let mvp: Matrix4<f32> = Matrix4::identity();
        let mvp_ref: &[f32; 16] = mvp.as_ref();
        let vertex_uniform_buffer = wgpu_context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: cast_slice(mvp_ref),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_uniforms = CameraUniforms::default();
        let camera_uniforms_buffer = wgpu_context.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: cast_slice(&[camera_uniforms]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );
        let camera_bind_group_layout = wgpu_context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::all(),
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("camera_bind_group_layout"),
        });
        let camera_bind_group = wgpu_context.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_uniforms_buffer.as_entire_binding(),
                }
            ],
            label: Some("camera_bind_group"),
        });

        let uniform_bind_group_layout = wgpu_context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
            label: Some("Uniform Bind Group Layout"),
        });

        let uniform_bind_group = wgpu_context.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: vertex_uniform_buffer.as_entire_binding(),
            }],
            label: Some("Uniform Bind Group"),
        });

        let pipeline_layout = wgpu_context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                &uniform_bind_group_layout,
                &camera_bind_group_layout,
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

        Material {
            shader_module: shader,
            vertex_uniform_buffer,

            camera_uniforms,
            camera_uniforms_buffer,
            camera_bind_group,

            uniform_bind_group_layout,
            uniform_bind_group,

            pipeline_layout,
            pipeline,
        }
    }
    pub fn new_default(wgpu_context: &mut WGPUContext) -> Material {
        Material::new(wgpu_context)
    }

    pub fn bind<'a, 'b>(&'a mut self, render_pass: &'b mut RenderPass<'a>, active_camera: &Camera, wgpu_context: &mut WGPUContext) {
        self.camera_uniforms.update(active_camera);
        wgpu_context.queue.write_buffer(&self.camera_uniforms_buffer, 0, cast_slice(&[self.camera_uniforms]));

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
    }
}