use futures_channel::oneshot;
use image::RgbaImage;
use std::future::Future;
use std::io;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Duration;

/// Starts copying an RGBA8 texture and returns its pixels asynchronously.
/// Accepts a single-sample, single-layer 2D texture with `COPY_SRC` usage.
/// Preserves the stored bytes without applying a color-space conversion.
/// Returns `Unsupported` on WebGL; browser readback requires WebGPU.
pub fn read_rgba8(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
) -> impl Future<Output = io::Result<RgbaImage>> + use<> {
    let readback = start_readback(device, queue, texture);
    async move { readback?.await }
}

fn start_readback(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
) -> io::Result<impl Future<Output = io::Result<RgbaImage>> + use<>> {
    #[cfg(target_arch = "wasm32")]
    if device.as_webgpu().is_none() {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Screenshots require WebGPU in browsers; WebGL readback is unsupported",
        ));
    }
    assert!(
        matches!(
            texture.format(),
            wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Rgba8UnormSrgb
        ),
        "Readback requires an RGBA8 texture"
    );
    let (width, height) = (texture.width(), texture.height());
    let row_bytes = width * 4;
    let padded_row_bytes =
        row_bytes.div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT) * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Image Readback"),
        size: u64::from(padded_row_bytes) * u64::from(texture.height()),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&Default::default());
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_row_bytes),
                rows_per_image: Some(texture.height()),
            },
        },
        texture.size(),
    );
    let _submission = queue.submit([encoder.finish()]);
    let (sender, receiver) = oneshot::channel();
    buffer
        .slice(..)
        .map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });
    #[cfg(not(target_arch = "wasm32"))]
    let device = device.clone();
    Ok(async move {
        #[cfg(not(target_arch = "wasm32"))]
        wait_for_mapping(device, _submission, receiver).await?;
        #[cfg(target_arch = "wasm32")]
        receiver
            .await
            .map_err(io::Error::other)?
            .map_err(io::Error::other)?;
        let pixels = {
            let view = buffer
                .slice(..)
                .get_mapped_range()
                .map_err(io::Error::other)?;
            view.chunks_exact(padded_row_bytes as usize)
                .flat_map(|row| row[..row_bytes as usize].iter().copied())
                .collect()
        };
        buffer.unmap();
        Ok(RgbaImage::from_raw(width, height, pixels)
            .expect("Readback rows match the image dimensions"))
    })
}

#[cfg(not(target_arch = "wasm32"))]
async fn wait_for_mapping(
    device: wgpu::Device,
    submission: wgpu::SubmissionIndex,
    receiver: oneshot::Receiver<Result<(), wgpu::BufferAsyncError>>,
) -> io::Result<()> {
    let (sender, polled) = oneshot::channel();
    // Native callbacks need polling; keep the wait off the caller's executor.
    std::thread::spawn(move || {
        let result = device.poll(wgpu::PollType::Wait {
            submission_index: Some(submission),
            timeout: Some(Duration::from_secs(10)),
        });
        let _ = sender.send(result);
    });
    polled
        .await
        .map_err(io::Error::other)?
        .map_err(io::Error::other)?;
    receiver
        .await
        .map_err(io::Error::other)?
        .map_err(io::Error::other)
}
