use wgpu::BufferAddress;
use crate::core::wgpu_context::WGPUContext;

pub fn create_buffer<T>(name: &str, wgpu_context: &mut WGPUContext) -> wgpu::Buffer {
    create_array_buffer::<T>(name, 1, wgpu_context)
}

pub fn create_array_buffer<T>(name: &str, nb_elements: u32, wgpu_context: &mut WGPUContext) -> wgpu::Buffer {
    wgpu_context.device.create_buffer(
        &wgpu::BufferDescriptor {
            label: Some(name),
            size: std::mem::size_of::<T>() as BufferAddress * nb_elements as BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }
    )
}