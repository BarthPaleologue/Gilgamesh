#![cfg(not(target_arch = "wasm32"))]

use gilgamesh::photometry::{Lux, Nits};
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
use glam::DVec3;
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
    let color_view = color.create_view(&wgpu::TextureViewDescriptor::default());
    let depth_view = depth.create_view(&wgpu::TextureViewDescriptor::default());
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
    constant.set_albedo_color(0.215_860_5, 0.215_860_5, 0.215_860_5);
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
            &primitive::plane(2, 4.0),
            Rc::new(material.into()),
            &gpu,
        )
    });
    let camera: Camera = Camera::default();
    camera.transform_mut().position = DVec3::new(0.0, 3.0, 3.0);
    camera.transform_mut().look_at(DVec3::ZERO, DVec3::Y);
    let light = DirectionalLight::default();
    let mut renderer = RasterRenderer::new(&gpu, wgpu::TextureFormat::Rgba8UnormSrgb);
    renderer.update_frame_bindings(
        &FrameBindingsData::new(&camera, std::slice::from_ref(&light), &[]),
        1.0,
        &gpu,
    );
    let mut images = Vec::new();
    for mesh in &meshes {
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        renderer.encode(
            [mesh],
            &gpu,
            &mut encoder,
            &RenderTarget {
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

#[test]
#[ignore = "GPU end-to-end check; run explicitly with approval"]
#[expect(
    clippy::too_many_lines,
    reason = "Keeps GPU fixture setup, execution and assertions together"
)]
fn roughness_and_normal_maps_match_equivalent_linear_surface_properties() {
    use gilgamesh::blocks::{rasterize, tone_map};
    use gilgamesh::core::graph_executor::GraphExecutor;
    use gilgamesh::core::render_graph::RenderGraph;
    use glam::Vec3;

    pollster::block_on(async {
        let mut executor = GraphExecutor::new().await;
        let camera: Camera = Camera::default();
        camera.transform_mut().position = DVec3::new(3.0, 3.0, 0.0);
        camera.transform_mut().look_at(DVec3::ZERO, DVec3::Y);
        let light = DirectionalLight::default();
        let mut target = executor.create_target(64, 64);
        let mut captures = Vec::new();
        for roughness in [32_u8, 128, 224] {
            for mapped in [false, true] {
                let mut material = PbrMaterial::new("Roughness");
                material.metallic = 0.0;
                material.set_albedo_color(0.3, 0.4, 0.5);
                material.roughness = f32::from(roughness) / 255.0;
                if mapped {
                    material.roughness = 0.99;
                    material.set_roughness_map(test_texture(
                        [roughness, 3, 240, 255],
                        executor.wgpu_context(),
                    ));
                }
                let mesh = Mesh::from_cpu_buffers(
                    "Plane",
                    &primitive::plane(2, 4.0),
                    Rc::new(material.into()),
                    executor.wgpu_context(),
                );
                let mut graph = RenderGraph::new();
                let scene = rasterize(
                    FrameBindingsData::new(&camera, std::slice::from_ref(&light), &[]),
                    [&mesh],
                    wgpu::TextureFormat::Rgba16Float,
                    &mut graph,
                );
                let output = tone_map(scene.color, wgpu::TextureFormat::Rgba8UnormSrgb, &mut graph);
                graph.export(output);
                executor.execute(graph, &mut target).unwrap();
                captures.push(target.read_image(executor.wgpu_context()).await.unwrap());
            }
        }
        for pair in captures.as_chunks::<2>().0 {
            assert!(pair[0].pixels().any(|p| p[0] > 0));
            for (a, b) in pair[0].as_raw().iter().zip(pair[1].as_raw()) {
                assert!(
                    a.abs_diff(*b) <= 1,
                    "Scalar roughness and linear red channel differ: {a} != {b}"
                );
            }
        }
        assert!(
            captures[2]
                .as_raw()
                .iter()
                .zip(captures[4].as_raw())
                .any(|(a, b)| a.abs_diff(*b) > 5),
            "Roughness must affect the rendered surface"
        );

        let encoded = [128_u8, 218, 218, 255];
        let tangent_normal = (Vec3::new(
            f32::from(encoded[0]),
            f32::from(encoded[1]),
            f32::from(encoded[2]),
        ) / 255.0
            * 2.0
            - Vec3::ONE)
            .normalize();
        let expected_normal =
            Vec3::new(tangent_normal.x, tangent_normal.z, -tangent_normal.y).to_array();
        let mut images = Vec::new();
        for mapped in [false, true] {
            let mut material = PbrMaterial::new("Normal");
            material.metallic = 0.0;
            material.roughness = 0.7;
            let mut geometry = primitive::plane(2, 4.0);
            if mapped {
                material.set_normal_map(test_texture(encoded, executor.wgpu_context()));
            } else {
                geometry.vertices.normals.fill(expected_normal);
            }
            let mesh = Mesh::from_cpu_buffers(
                "Plane",
                &geometry,
                Rc::new(material.into()),
                executor.wgpu_context(),
            );
            let mut graph = RenderGraph::new();
            let scene = rasterize(
                FrameBindingsData::new(&camera, std::slice::from_ref(&light), &[]),
                [&mesh],
                wgpu::TextureFormat::Rgba16Float,
                &mut graph,
            );
            let output = tone_map(scene.color, wgpu::TextureFormat::Rgba8UnormSrgb, &mut graph);
            graph.export(output);
            executor.execute(graph, &mut target).unwrap();
            images.push(target.read_image(executor.wgpu_context()).await.unwrap());
        }
        assert!(images[0].pixels().any(|p| p[0] > 0));
        for (a, b) in images[0].as_raw().iter().zip(images[1].as_raw()) {
            assert!(
                a.abs_diff(*b) <= 1,
                "Linear tangent-space normal differs from equivalent geometry: {a} != {b}"
            );
        }
    });
}

fn test_texture(pixel: [u8; 4], gpu: &WGPUContext) -> Rc<Texture<Linear>> {
    let mut png = Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(1, 1, image::Rgba(pixel)))
        .write_to(&mut png, image::ImageFormat::Png)
        .expect("Failed to encode the test texture as PNG");
    Rc::new(Texture::<Linear>::new("PBR data", png.get_ref(), gpu))
}

#[test]
#[ignore = "GPU end-to-end check; run explicitly with approval"]
fn emissive_textures_multiply_linear_hdr_factors_without_received_light() {
    use gilgamesh::blocks::{rasterize, tone_map};
    use gilgamesh::core::graph_executor::GraphExecutor;
    use gilgamesh::core::render_graph::RenderGraph;
    use gilgamesh::texture::ColorTexture;

    pollster::block_on(async {
        let mut executor = GraphExecutor::new().await;
        let camera: Camera = Camera::default();
        camera.transform_mut().position = DVec3::new(0.0, 3.0, 3.0);
        camera.transform_mut().look_at(DVec3::ZERO, DVec3::Y);
        let mut light = DirectionalLight::default();
        light.set_illuminance(Lux::new(0.0));
        let mut target = executor.create_target(32, 32);
        let factor = [8.0, 2.0, 0.5];
        let mut png = Cursor::new(Vec::new());
        image::DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(
            1,
            1,
            image::Rgba([128, 128, 128, 17]),
        ))
        .write_to(&mut png, image::ImageFormat::Png)
        .unwrap();
        for linear in [false, true] {
            let (texture, gray): (ColorTexture, f32) = if linear {
                (
                    Rc::new(Texture::<Linear>::new(
                        "Linear emission",
                        png.get_ref(),
                        executor.wgpu_context(),
                    ))
                    .into(),
                    128.0 / 255.0,
                )
            } else {
                (
                    Rc::new(Texture::<Srgb>::new(
                        "sRGB emission",
                        png.get_ref(),
                        executor.wgpu_context(),
                    ))
                    .into(),
                    0.215_860_5,
                )
            };
            let mut reference = PbrMaterial::new("Constant emission");
            reference.set_emissive_luminance(Nits::new(1.0));
            reference.set_emissive_color(factor[0] * gray, factor[1] * gray, factor[2] * gray);
            let mut textured = PbrMaterial::new("Textured emission");
            textured.set_emissive_texture(texture);
            textured.set_emissive_luminance(Nits::new(1.0));
            textured.set_emissive_color(factor[0], factor[1], factor[2]);
            let mut disabled = textured.clone();
            disabled.set_emissive_luminance(Nits::new(1.0));
            disabled.set_emissive_color(0.0, 0.0, 0.0);
            let mut images = Vec::new();
            for material in [reference, textured, disabled] {
                let mesh = Mesh::from_cpu_buffers(
                    "Emitter",
                    &primitive::plane(2, 4.0),
                    Rc::new(material.into()),
                    executor.wgpu_context(),
                );
                let mut graph = RenderGraph::new();
                let scene = rasterize(
                    FrameBindingsData::new(&camera, std::slice::from_ref(&light), &[]),
                    [&mesh],
                    wgpu::TextureFormat::Rgba16Float,
                    &mut graph,
                );
                let output = tone_map(scene.color, wgpu::TextureFormat::Rgba8UnormSrgb, &mut graph);
                graph.export(output);
                executor.execute(graph, &mut target).unwrap();
                images.push(target.read_image(executor.wgpu_context()).await.unwrap());
            }
            let center = images[1].get_pixel(16, 16);
            assert!(
                center[0] > 200 && center[1] > center[2],
                "Emission must remain HDR and keep its color without lights: {center:?}"
            );
            assert_eq!(center[3], 255, "Emissive texture alpha must be ignored");
            for (a, b) in images[0].as_raw().iter().zip(images[1].as_raw()) {
                assert!(
                    a.abs_diff(*b) <= 1,
                    "linear={linear}: constant {a} != textured {b}"
                );
            }
            assert_eq!(
                images[2].get_pixel(16, 16).0,
                [0, 0, 0, 255],
                "A zero emission factor must disable the texture"
            );
        }
    });
}
