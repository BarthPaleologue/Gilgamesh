#![cfg(not(target_arch = "wasm32"))]

use gilgamesh::core::color_pass::ColorPass;
mod common;

use common::read_rgba8;
use wgpu::util::{DeviceExt, TextureDataOrder};

#[test]
#[ignore = "GPU end-to-end check; run explicitly with approval"]
fn khronos_neutral_and_blit_preserve_colors_without_double_gamma() {
    let (device, queue) = pollster::block_on(async {
        let adapter = wgpu::Instance::default()
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .unwrap();
        adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .unwrap()
    });
    let hdr_pixel_bits: [[u16; 4]; 12] = [
        [0x4400, 0x3c00, 0x3400, 0x3c00],
        [0x0000, 0x3800, 0x4000, 0x3800],
        [0x4800, 0x0000, 0x3c00, 0x3c00],
        [0x1800, 0x1c00, 0x4400, 0x3c00],
        [0x3c00, 0x4000, 0x0000, 0x3c00],
        [0x3400, 0x4800, 0x3800, 0x3c00],
        [0x0000, 0x0000, 0x0000, 0x3c00],
        [0xbc00, 0xb800, 0xb400, 0x3400],
        [0x2400, 0x2400, 0x2400, 0x3800],
        [0x2800, 0x2800, 0x2800, 0x3c00],
        [0x3a00, 0x0000, 0x3400, 0x3c00],
        [0x3a20, 0x0000, 0x3400, 0x3c00],
    ];
    // Khronos reference results, encoded to sRGB with alpha unchanged.
    let expected: [[u8; 4]; 12] = [
        [253, 182, 158, 255],
        [101, 157, 251, 128],
        [254, 189, 199, 255],
        [150, 150, 253, 255],
        [195, 251, 101, 255],
        [191, 254, 193, 255],
        [0, 0, 0, 255],
        [0, 0, 0, 64],
        [5, 5, 5, 128],
        [18, 18, 18, 255],
        [225, 0, 137, 255],
        [227, 0, 137, 255],
    ];
    let tone_mapping = ColorPass::tone_mapping(&device, wgpu::TextureFormat::Rgba8UnormSrgb);
    for (width, height) in [(4, 3), (3, 4)] {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let hdr = device.create_texture_with_data(
            &queue,
            &wgpu::TextureDescriptor {
                label: Some("HDR Test Input"),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba16Float,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
            TextureDataOrder::LayerMajor,
            bytemuck::cast_slice(&hdr_pixel_bits),
        );
        let create_output = |format| {
            device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Color Test Output"),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            })
        };
        let ldr = create_output(wgpu::TextureFormat::Rgba8UnormSrgb);
        let ldr_view = ldr.create_view(&wgpu::TextureViewDescriptor::default());
        let hdr_source = tone_mapping.bind_source(
            &device,
            &hdr.create_view(&wgpu::TextureViewDescriptor::default()),
        );
        for format in [
            wgpu::TextureFormat::Rgba8UnormSrgb,
            wgpu::TextureFormat::Rgba8Unorm,
        ] {
            let output = create_output(format);
            let image_blit = ColorPass::image_blit(&device, format);
            let ldr_source = image_blit.bind_source(&device, &ldr_view);
            let mut encoder =
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
            tone_mapping.encode(&mut encoder, &hdr_source, &ldr_view);
            image_blit.encode(
                &mut encoder,
                &ldr_source,
                &output.create_view(&wgpu::TextureViewDescriptor::default()),
            );
            queue.submit([encoder.finish()]);
            for texture in [&ldr, &output] {
                let actual = read_rgba8(&device, &queue, texture);
                assert_eq!(actual.len(), expected.len() * 4);
                for (index, (&actual, &expected)) in actual
                    .iter()
                    .zip(bytemuck::cast_slice::<_, u8>(&expected))
                    .enumerate()
                {
                    assert!(
                        actual.abs_diff(expected) <= 1,
                        "{width}x{height} {format:?}, byte {index}: {actual} != {expected}"
                    );
                }
            }
        }
    }
}
