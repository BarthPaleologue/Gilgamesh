#![cfg(not(target_arch = "wasm32"))]

mod common;

use common::read_rgba8;
use gilgamesh::camera::Camera;
use gilgamesh::core::raster_renderer::RasterRenderer;
use gilgamesh::core::render_context::FrameBindingsData;
use gilgamesh::core::render_target::RenderTarget;
use gilgamesh::core::wgpu_context::WGPUContext;
use gilgamesh::geometry::{mesh::Mesh, primitive};
use gilgamesh::lights::directional_light::DirectionalLight;
use gilgamesh::materials::pbr::PbrMaterial;
use gilgamesh::texture::{Linear, Srgb, Texture};
use gilgamesh::transform::Transformable;
use glam::{DVec3, dcamera::rh};
use std::io::Cursor;
use std::rc::Rc;

#[test]
#[ignore = "GPU end-to-end check; run explicitly with approval"]
fn albedo_encodings_match_equivalent_linear_constants() {
    let gpu = pollster::block_on(WGPUContext::new(wgpu::Instance::default(), None));
    let size = wgpu::Extent3d {
        width: 32,
        height: 32,
        depth_or_array_layers: 1,
    };
    let attachment = |format, usage| {
        gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("PBR Sampling Test"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage,
            view_formats: &[],
        })
    };
    let color = attachment(
        wgpu::TextureFormat::Rgba8UnormSrgb,
        wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
    );
    let depth = attachment(
        wgpu::TextureFormat::Depth32Float,
        wgpu::TextureUsages::RENDER_ATTACHMENT,
    );
    let color_view = color.create_view(&Default::default());
    let depth_view = depth.create_view(&Default::default());
    let mut png = Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(
        1,
        1,
        image::Rgba([128, 128, 128, 255]),
    ))
    .write_to(&mut png, image::ImageFormat::Png)
    .unwrap();
    let texture = Rc::new(Texture::<Srgb>::new("Gray", png.get_ref(), &gpu));
    let mut textured = PbrMaterial::new("Textured");
    textured.set_albedo_texture(texture);
    let mut constant = PbrMaterial::new("Constant");
    // sRGB byte 128 represents approximately 0.2158605 in linear space.
    constant.set_albedo_color(0.2158605, 0.2158605, 0.2158605);
    let mut linear_textured = PbrMaterial::new("Linear Textured");
    linear_textured.set_albedo_texture(Rc::new(Texture::<Linear>::new(
        "Linear Gray",
        png.get_ref(),
        &gpu,
    )));
    let mut linear_constant = PbrMaterial::new("Linear Constant");
    let linear_gray = 128.0 / 255.0;
    linear_constant.set_albedo_color(linear_gray, linear_gray, linear_gray);
    let meshes = [constant, textured, linear_constant, linear_textured].map(|material| {
        Mesh::from_cpu_buffers(
            "Albedo",
            primitive::plane(2, 4.0),
            Rc::new(material.into()),
            &gpu,
        )
    });
    let camera: Camera = Camera::default();
    let position = DVec3::new(0.0, 3.0, 3.0);
    camera.transform_mut().position = position;
    camera.transform_mut().rotation =
        rh::view::look_at_quat(position, DVec3::ZERO, DVec3::Y).conjugate();
    let light = DirectionalLight::default();
    let mut renderer = RasterRenderer::new(&gpu, wgpu::TextureFormat::Rgba8UnormSrgb);
    renderer.update_frame_bindings(FrameBindingsData::new(&camera, &light, &[]), &gpu);
    let mut images = Vec::new();
    for mesh in &meshes {
        let mut encoder = gpu.device.create_command_encoder(&Default::default());
        renderer.encode(
            [mesh],
            &gpu,
            &mut encoder,
            RenderTarget {
                color: &color_view,
                depth: &depth_view,
            },
        );
        gpu.queue.submit([encoder.finish()]);
        images.push(read_rgba8(&gpu.device, &gpu.queue, &color));
    }
    let center = (16 * 32 + 16) * 4;
    let (paired_images, _) = images.as_chunks::<2>();
    for (encoding, pair) in ["Srgb", "Linear"].into_iter().zip(paired_images) {
        assert!(pair[0][center] > 0);
        assert!(pair[0][center].abs_diff(pair[0][center + 2]) <= 1);
        for (index, (&constant, &textured)) in pair[0].iter().zip(&pair[1]).enumerate() {
            assert!(
                constant.abs_diff(textured) <= 1,
                "{encoding}, byte {index}: constant {constant} != textured {textured}"
            );
        }
    }
}
