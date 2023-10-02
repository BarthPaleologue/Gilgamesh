use std::cell::Ref;
use std::num::NonZeroU32;
use std::ops::Deref;
use bytemuck::{cast_slice};
use load_file::load_str;
use wgpu::{BindGroup, BindGroupLayout, Buffer, PipelineLayout, RenderPass, RenderPipeline, ShaderModule};
use wgpu::util::{DeviceExt};
use crate::camera::camera::Camera;
use crate::camera::uniforms::CameraUniforms;

use crate::geometry::mesh::Vertex;
use crate::core::wgpu_context::WGPUContext;
use crate::lights::directional_light::{DirectionalLight, DirectionalLightUniform};
use crate::lights::point_light::{PointLight, PointLightUniforms};
use crate::settings::MAX_POINT_LIGHTS;
use crate::transform::{Transform, TransformUniforms};

pub struct Material {
    pub shader_module: ShaderModule,

    pub transform_uniforms: TransformUniforms,
    pub transform_uniforms_buffer: Buffer,
    pub transform_bind_group: BindGroup,

    pub camera_uniforms: CameraUniforms,
    pub camera_uniforms_buffer: Buffer,

    pub light_uniforms: DirectionalLightUniform,
    pub light_uniforms_buffer: Buffer,

    pub point_light_uniforms: [PointLightUniforms; MAX_POINT_LIGHTS],
    pub point_light_buffer: Buffer,
    pub nb_point_lights_buffer: Buffer,

    pub uniform_bind_group: BindGroup,

    pub pipeline_layout: PipelineLayout,
    pub pipeline: RenderPipeline,
}

impl Material {
    pub fn new(shader_file: &str, wgpu_context: &mut WGPUContext) -> Material {
        // load shader from file at runtime
        let shader_string = load_str!(shader_file);
        let shader = wgpu_context.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_string.into()),
        });

        let transform_uniforms = TransformUniforms::default();
        let transform_uniforms_buffer = wgpu_context.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Transform Buffer"),
                contents: cast_slice(&[transform_uniforms]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let camera_uniforms = CameraUniforms::default();
        let camera_uniforms_buffer = wgpu_context.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: cast_slice(&[camera_uniforms]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let light_uniforms = DirectionalLightUniform::default();
        let light_uniforms_buffer = wgpu_context.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Light Buffer"),
                contents: cast_slice(&[light_uniforms]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let point_light_uniforms = [PointLightUniforms::default(); MAX_POINT_LIGHTS];
        let point_light_storage_buffer = wgpu_context.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Point Light Storage Buffer"),
                contents: cast_slice(&[point_light_uniforms]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let nb_point_lights_buffer = wgpu_context.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Number of Point Lights Buffer"),
                contents: cast_slice(&[0u32]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        // the bare minimum to make a material
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

        let uniform_bind_group_layout = wgpu_context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }, wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: NonZeroU32::new(MAX_POINT_LIGHTS as u32),
            }, wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::FRAGMENT,
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
                resource: camera_uniforms_buffer.as_entire_binding(),
            }, wgpu::BindGroupEntry {
                binding: 1,
                resource: light_uniforms_buffer.as_entire_binding(),
            }, wgpu::BindGroupEntry {
                binding: 2,
                resource: point_light_storage_buffer.as_entire_binding(),
            }, wgpu::BindGroupEntry {
                binding: 3,
                resource: nb_point_lights_buffer.as_entire_binding(),
            }],
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

        Material {
            shader_module: shader,

            transform_uniforms,
            transform_uniforms_buffer,
            transform_bind_group,

            camera_uniforms,
            camera_uniforms_buffer,

            light_uniforms,
            light_uniforms_buffer,

            point_light_uniforms,
            point_light_buffer: point_light_storage_buffer,
            nb_point_lights_buffer,

            uniform_bind_group,

            pipeline_layout,
            pipeline,
        }
    }
    pub fn new_default(wgpu_context: &mut WGPUContext) -> Material {
        Material::new("../shader/default.wgsl", wgpu_context)
    }

    pub fn bind<'a, 'b>(&'a mut self, render_pass: &'b mut RenderPass<'a>, transform: Ref<Transform>, active_camera: &Camera, point_lights: &[PointLight], directional_light: &DirectionalLight, wgpu_context: &mut WGPUContext) {
        self.transform_uniforms.update(transform.deref());
        wgpu_context.queue.write_buffer(&self.transform_uniforms_buffer, 0, cast_slice(&[self.transform_uniforms]));

        self.camera_uniforms.update(active_camera);
        wgpu_context.queue.write_buffer(&self.camera_uniforms_buffer, 0, cast_slice(&[self.camera_uniforms]));

        self.light_uniforms.update(directional_light);
        wgpu_context.queue.write_buffer(&self.light_uniforms_buffer, 0, cast_slice(&[self.light_uniforms]));

        for i in 0..point_lights.len() {
            self.point_light_uniforms[i].update(&point_lights[i]);
        }
        wgpu_context.queue.write_buffer(&self.point_light_buffer, 0, cast_slice(&[self.point_light_uniforms]));
        wgpu_context.queue.write_buffer(&self.nb_point_lights_buffer, 0, cast_slice(&[point_lights.len() as u32]));

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.transform_bind_group, &[]);
        render_pass.set_bind_group(1, &self.uniform_bind_group, &[]);
    }
}