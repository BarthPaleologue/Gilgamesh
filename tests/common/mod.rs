pub(crate) fn read_rgba8(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
) -> Vec<u8> {
    pollster::block_on(gilgamesh::core::readback::read_rgba8(
        device, queue, texture,
    ))
    .expect("Failed to read the test texture")
    .into_raw()
}
