use std::cell::Ref;
use std::ops::Deref;
use std::rc::Rc;
use std::borrow::Borrow;
use bytemuck::{cast_slice};
use load_file::load_str;
use wgpu::{BindGroup, BindGroupLayout, Buffer, PipelineLayout, RenderPass, RenderPipeline, ShaderModule};
use crate::camera::camera::Camera;
use crate::camera::uniforms::CameraUniforms;

use crate::geometry::mesh::Vertex;
use crate::core::wgpu_context::WGPUContext;
use crate::materials::utils::{create_buffer};
use crate::texture::Texture;
use crate::transform::{Transform, TransformUniforms};

pub struct MaterialPipeline {
    pub shader_module: ShaderModule,

    pub transform_uniforms: TransformUniforms,
    pub transform_uniforms_buffer: Buffer,

    pub camera_uniforms: CameraUniforms,
    pub camera_uniforms_buffer: Buffer,

    pub required_bind_group: BindGroup,

    pub uniform_bind_group: BindGroup,

    pub texture_bind_group: BindGroup,

    pub pipeline_layout: PipelineLayout,
    pub pipeline: RenderPipeline,
}

impl MaterialPipeline {
    pub fn new(shader_file: &str, uniforms: &[&Buffer], textures: &[&Texture], polygon_mode: wgpu::PolygonMode, back_face_culling: bool, wgpu_context: &mut WGPUContext) -> MaterialPipeline {
        // load shader from file at runtime
        let shader_string = load_str!(shader_file);
        let shader = wgpu_context.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_string.into()),
        });

        let transform_uniforms = TransformUniforms::default();
        let transform_uniforms_buffer = create_buffer::<TransformUniforms>("Transform Buffer", wgpu_context);

        let camera_uniforms = CameraUniforms::default();
        let camera_uniforms_buffer = create_buffer::<CameraUniforms>("Camera Buffer", wgpu_context);

        let required_bind_group_layout = wgpu_context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Required Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::all(),
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }, wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::all(),
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let required_bind_group = wgpu_context.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &required_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: transform_uniforms_buffer.as_entire_binding(),
            }, wgpu::BindGroupEntry {
                binding: 1,
                resource: camera_uniforms_buffer.as_entire_binding(),
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

        let entries: Vec<wgpu::BindGroupLayoutEntry> = textures.iter().enumerate().map(|(i, texture)| {
            texture.create_bind_group_layout_entries(2 * i as u32)
        }).flatten().collect();
        let texture_bind_group_layout = wgpu_context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &entries,
            label: Some("Texture Bind Group Layout"),
        });

        let entries: Vec<wgpu::BindGroupEntry> = textures.iter().enumerate().map(|(i, texture)| {
            texture.create_bind_group_entries(2 * i as u32)
        }).flatten().collect();
        let texture_bind_group = wgpu_context.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &entries,
            label: Some("Texture Bind Group"),
        });

        let pipeline_layout = wgpu_context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                &required_bind_group_layout,
                &uniform_bind_group_layout,
                &texture_bind_group_layout
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
                polygon_mode,
                cull_mode: if back_face_culling { Some(wgpu::Face::Back) } else { None },
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

            camera_uniforms,
            camera_uniforms_buffer,

            required_bind_group,

            uniform_bind_group,

            texture_bind_group,

            pipeline_layout,
            pipeline,
        }
    }

    pub fn bind<'a, 'b>(&'a mut self, render_pass: &'b mut RenderPass<'a>, transform: Ref<Transform>, active_camera: &Camera, wgpu_context: &mut WGPUContext) {
        self.transform_uniforms.update(transform.deref());
        wgpu_context.queue.write_buffer(&self.transform_uniforms_buffer, 0, cast_slice(&[self.transform_uniforms]));

        self.camera_uniforms.update(active_camera);
        wgpu_context.queue.write_buffer(&self.camera_uniforms_buffer, 0, cast_slice(&[self.camera_uniforms]));

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.required_bind_group, &[]);
        render_pass.set_bind_group(1, &self.uniform_bind_group, &[]);
        render_pass.set_bind_group(2, &self.texture_bind_group, &[]);
    }
}