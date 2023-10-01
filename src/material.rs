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
    pub fragment_uniform_buffer: Buffer,

    pub camera_uniforms: CameraUniforms,
    pub camera_uniforms_buffer: Buffer,

    pub uniform_bind_group_layout: BindGroupLayout,
    pub uniform_bind_group: BindGroup,

    pub pipeline_layout: PipelineLayout,
    pub pipeline: RenderPipeline,
}

impl Material {
    pub fn new(shader: ShaderModule, vertex_uniform_buffer: Buffer, fragment_uniform_buffer: Buffer, wgpu_context: &mut WGPUContext) -> Material {
        let camera_uniforms = CameraUniforms::default();
        let camera_uniforms_buffer = wgpu_context.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: cast_slice(&[camera_uniforms]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

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
                visibility: wgpu::ShaderStages::all(),
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
            }, wgpu::BindGroupEntry {
                binding: 1,
                resource: fragment_uniform_buffer.as_entire_binding(),
            }, wgpu::BindGroupEntry {
                binding: 2,
                resource: camera_uniforms_buffer.as_entire_binding(),
            }],
            label: Some("Uniform Bind Group"),
        });

        let pipeline_layout = wgpu_context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&uniform_bind_group_layout],
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
            fragment_uniform_buffer,

            camera_uniforms,
            camera_uniforms_buffer,

            uniform_bind_group_layout,
            uniform_bind_group,

            pipeline_layout,
            pipeline,
        }
    }
    pub fn new_default(wgpu_context: &mut WGPUContext) -> Material {
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

        let fragment_uniform_buffer = wgpu_context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Fragment Uniform Buffer"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Material::new(shader, vertex_uniform_buffer, fragment_uniform_buffer, wgpu_context)
    }

    pub fn new_2d_terrain(max_height: f32, wgpu_context: &mut WGPUContext) -> Material {
        let shader = wgpu_context.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader/flat_terrain.wgsl").into()),
        });

        let mvp: Matrix4<f32> = Matrix4::identity();
        let mvp_ref: &[f32; 16] = mvp.as_ref();
        let vertex_uniform_buffer = wgpu_context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: cast_slice(mvp_ref),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // create fragment uniform buffer. here we set eye_position = camera_position and light_position = eye_position
        let fragment_uniform_buffer = wgpu_context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Fragment Uniform Buffer"),
            size: 32,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // store light and eye positions
        let light_dir: &[f32; 3] = &[1.0, 1.0, 0.5];
        let camera_position: &[f32; 3] = &[0.0, 0.0, 0.0];
        wgpu_context.queue.write_buffer(&fragment_uniform_buffer, 0, cast_slice(light_dir));
        wgpu_context.queue.write_buffer(&fragment_uniform_buffer, 12, cast_slice(camera_position));
        wgpu_context.queue.write_buffer(&fragment_uniform_buffer, 28, cast_slice(&max_height.to_ne_bytes()));

        /*#[repr(C)]
        #[derive(Debug, Copy, Clone, Pod, Zeroable)]
        struct FragUniforms {
            light_dir: [f32; 3],
            camera_position: [f32; 3],
            max_height: f32,
        }
        let frag_uniforms = FragUniforms {
            light_dir: [1.0, 1.0, 0.5],
            camera_position: [0.0, 0.0, 0.0],
            max_height,
        };*/

        // create fragment uniform buffer. here we set eye_position = camera_position and light_position = eye_position
        /*let fragment_uniform_buffer = wgpu_context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Fragment Uniform Buffer"),
            contents: cast_slice(&[frag_uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });*/

        Material::new(shader, vertex_uniform_buffer, fragment_uniform_buffer, wgpu_context)
    }

    pub fn new_sphere_terrain(sphere_radius: f32, max_height: f32, wgpu_context: &mut WGPUContext) -> Material {
        let shader = wgpu_context.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader/sphere_terrain.wgsl").into()),
        });

        let mvp: Matrix4<f32> = Matrix4::identity();
        let mvp_ref: &[f32; 16] = mvp.as_ref();
        let vertex_uniform_buffer = wgpu_context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: cast_slice(mvp_ref),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // store light and eye positions
        /*#[repr(C)]
        #[derive(Debug, Copy, Clone, Pod, Zeroable)]
        struct FragUniforms {
            light_dir: [f32; 3],
            camera_position: [f32; 3],
            max_height: f32,
            sphere_radius: f32
        }
        let frag_uniforms = FragUniforms {
            light_dir: [1.0, 1.0, 0.5],
            camera_position: [0.0, 0.0, 0.0],
            max_height,
            sphere_radius,
        };*/

        // create fragment uniform buffer. here we set eye_position = camera_position and light_position = eye_position
        let fragment_uniform_buffer = wgpu_context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Fragment Uniform Buffer"),
            size: 48,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let light_dir: &[f32; 3] = &[1.0, 1.0, 0.5];
        let camera_position: &[f32; 3] = &[0.0, 0.0, 0.0];
        wgpu_context.queue.write_buffer(&fragment_uniform_buffer, 0, cast_slice(light_dir));
        wgpu_context.queue.write_buffer(&fragment_uniform_buffer, 12, cast_slice(camera_position));
        wgpu_context.queue.write_buffer(&fragment_uniform_buffer, 28, cast_slice(&max_height.to_ne_bytes()));
        wgpu_context.queue.write_buffer(&fragment_uniform_buffer, 32, cast_slice(&sphere_radius.to_ne_bytes()));

        // create fragment uniform buffer. here we set eye_position = camera_position and light_position = eye_position
        /*let fragment_uniform_buffer = wgpu_context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Fragment Uniform Buffer"),
            contents: cast_slice(&[]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });*/

        Material::new(shader, vertex_uniform_buffer, fragment_uniform_buffer, wgpu_context)
    }

    pub fn bind<'a, 'b>(&'a mut self, render_pass: &'b mut RenderPass<'a>, active_camera: &Camera, wgpu_context: &mut WGPUContext) {
        self.camera_uniforms.update(active_camera);
        wgpu_context.queue.write_buffer(&self.camera_uniforms_buffer, 0, cast_slice(&[self.camera_uniforms]));

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
    }
}