use crate::core::uniforms::{create_buffer, uniform_layout_entry};
use crate::core::wgpu_context::WGPUContext;
use crate::transform::{Transform, TransformUniforms};

pub(crate) struct ObjectBindings {
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl ObjectBindings {
    pub(crate) fn create_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Object Bind Group Layout"),
            entries: &[uniform_layout_entry(0)],
        })
    }

    pub(crate) fn new(name: &str, layout: &wgpu::BindGroupLayout, ctx: &WGPUContext) -> Self {
        let buffer = create_buffer::<TransformUniforms>(&format!("{name} Transform Buffer"), ctx);
        let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Object Bind Group"),
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });
        Self { buffer, bind_group }
    }

    pub(crate) fn update(&self, transform: &Transform, queue: &wgpu::Queue) {
        let mut uniforms = TransformUniforms::default();
        uniforms.update(transform);
        queue.write_buffer(&self.buffer, 0, bytemuck::bytes_of(&uniforms));
    }

    pub(crate) fn bind(&self, pass: &mut wgpu::RenderPass<'_>) {
        pass.set_bind_group(2, &self.bind_group, &[]);
    }
}
