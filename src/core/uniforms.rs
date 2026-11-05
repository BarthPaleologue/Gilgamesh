use crate::core::wgpu_context::WGPUContext;
use wgpu::BufferAddress;

pub(crate) fn create_buffer<T>(name: &str, wgpu_context: &WGPUContext) -> wgpu::Buffer {
    create_array_buffer::<T>(name, 1, wgpu_context)
}

pub(crate) fn create_array_buffer<T>(
    name: &str,
    nb_elements: usize,
    wgpu_context: &WGPUContext,
) -> wgpu::Buffer {
    wgpu_context.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(name),
        size: std::mem::size_of::<T>() as BufferAddress * nb_elements as BufferAddress,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

pub(crate) fn uniform_layout_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}
