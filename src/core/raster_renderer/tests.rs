use super::*;
use crate::camera::{Camera, Orthographic};
use crate::geometry::primitive;
use crate::lights::directional_light::DirectionalLight;
use crate::materials::blinn_phong::BlinnPhongMaterial;
use crate::materials::pbr::PbrMaterial;
use crate::photometry::{Lux, Nits};
use glam::DVec3;

#[test]
#[ignore = "GPU end-to-end check; run explicitly with approval"]
#[expect(
    clippy::too_many_lines,
    reason = "Keeps GPU fixture setup, execution and assertions together"
)]
fn shared_materials_have_renderer_local_runtime() {
    let gpu = pollster::block_on(WGPUContext::new(wgpu::Instance::default(), None));
    let camera = Camera::new(Orthographic {
        size_vertical: 4.0,
        near: 0.1,
        far: 10.0,
    });
    camera.transform_mut().position = DVec3::new(0.0, 3.0, 0.0);
    camera.transform_mut().look_at(DVec3::ZERO, DVec3::NEG_Z);
    let mut light = DirectionalLight::default();
    light.set_illuminance(Lux::new(0.0));
    let depth = attachment(&gpu.device, wgpu::TextureFormat::Depth32Float);

    for mut description in [
        Material::from(PbrMaterial::new("PBR")),
        Material::from(BlinnPhongMaterial::new("Blinn-Phong")),
    ] {
        set_unlit_color(&mut description, [1.0, 0.0, 0.0]);
        let material = Rc::new(description);
        let source = Rc::downgrade(&material);
        let mut left =
            Mesh::from_cpu_buffers("Left", &primitive::plane(2, 1.0), material.clone(), &gpu);
        let right =
            Mesh::from_cpu_buffers("Right", &primitive::plane(2, 1.0), material.clone(), &gpu);
        left.transform_mut().position.x = -1.0;
        right.transform_mut().position.x = 1.0;
        let left_transform = Rc::downgrade(&left.transform_rc());
        let right_transform = Rc::downgrade(&right.transform_rc());
        let mut renderers = [
            wgpu::TextureFormat::Rgba8UnormSrgb,
            wgpu::TextureFormat::Rgba8Unorm,
        ]
        .map(|format| {
            let mut renderer = RasterRenderer::new(&gpu, format);
            renderer.update_frame_bindings(
                &FrameBindingsData::new(&camera, std::slice::from_ref(&light), &[]),
                1.0,
                &gpu,
            );
            (renderer, attachment(&gpu.device, format))
        });

        for (renderer, color) in &mut renderers {
            let image = render(&gpu, renderer, &[&left, &right], color, &depth);
            assert_channel(&image, 16, 0);
            assert_channel(&image, 48, 0);
            assert_eq!(renderer.materials.len(), 1);
            assert_eq!(renderer.transforms.len(), 2);
        }
        let (renderer, color) = &mut renderers[0];
        left.transform_mut().position.x = 0.0;
        let image = render(&gpu, renderer, &[&left, &right], color, &depth);
        assert_channel(&image, 32, 0);
        assert!(pixel(&image, 16)[2] > pixel(&image, 16)[0]);
        assert_eq!(renderer.materials.len(), 1);
        assert_eq!(renderer.transforms.len(), 2);

        let mut replacement = material.as_ref().clone();
        set_unlit_color(&mut replacement, [0.0, 1.0, 0.0]);
        left.set_material(Rc::new(replacement));
        let image = render(&gpu, renderer, &[&left, &right], color, &depth);
        assert_channel(&image, 32, 1);
        assert_channel(&image, 48, 0);
        assert_eq!(renderer.materials.len(), 2);

        drop(left);
        drop(right);
        let mut material = material;
        set_unlit_color(Rc::make_mut(&mut material), [0.0, 0.0, 1.0]);
        assert!(source.upgrade().is_none());
        let mesh = Mesh::from_cpu_buffers("Edited", &primitive::plane(2, 1.0), material, &gpu);
        for (renderer, color) in &mut renderers {
            let image = render(&gpu, renderer, &[&mesh], color, &depth);
            assert_channel(&image, 32, 2);
            assert_eq!(renderer.materials.len(), 1);
            assert_eq!(renderer.transforms.len(), 1);

            renderer.set_polygon_mode(wgpu::PolygonMode::Line);
            let wireframe = render(&gpu, renderer, &[&mesh], color, &depth);
            let blue_pixels = |pixels: &[u8]| {
                pixels
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .filter(|pixel| pixel[0] == 0 && pixel[1] == 0 && pixel[2] > 100)
                    .count()
            };
            assert!(blue_pixels(&wireframe) > 0);
            assert!(blue_pixels(&wireframe) < blue_pixels(&image));
            renderer.set_polygon_mode(wgpu::PolygonMode::Fill);
            assert_eq!(render(&gpu, renderer, &[&mesh], color, &depth), image);
        }
        assert!(left_transform.upgrade().is_none());
        assert!(right_transform.upgrade().is_none());

        let mut description = mesh.material().as_ref().clone();
        match &mut description {
            Material::Pbr(material) => material.set_double_sided(true),
            Material::BlinnPhong(material) => material.set_double_sided(true),
        }
        let mut mesh = mesh;
        mesh.transform_mut().rotation = glam::DQuat::from_rotation_x(std::f64::consts::PI);
        let (renderer, color) = &mut renderers[0];
        let culled = render(&gpu, renderer, &[&mesh], color, &depth);
        assert!(pixel(&culled, 32)[0] > 0);
        mesh.set_material(Rc::new(description));
        let visible = render(&gpu, renderer, &[&mesh], color, &depth);
        assert_channel(&visible, 32, 2);

        let source = Rc::downgrade(mesh.material());
        drop(mesh);
        assert!(source.upgrade().is_none());
        for (renderer, color) in &mut renderers {
            render(&gpu, renderer, &[], color, &depth);
            assert!(renderer.materials.is_empty());
            assert!(renderer.transforms.is_empty());
        }
    }
}

#[test]
#[ignore = "GPU end-to-end check; run explicitly with approval"]
fn directional_lights_accumulate_and_can_be_removed_between_frames() {
    let gpu = pollster::block_on(WGPUContext::new(wgpu::Instance::default(), None));
    let camera = Camera::new(Orthographic {
        size_vertical: 4.0,
        near: 0.1,
        far: 10.0,
    });
    camera.transform_mut().position = DVec3::new(0.0, 3.0, 0.0);
    camera.transform_mut().look_at(DVec3::ZERO, DVec3::NEG_Z);
    let lights = [0.2, 0.3].map(|intensity| {
        let mut light = DirectionalLight::default();
        light.set_illuminance(Lux::new(intensity));
        light
    });
    let color = attachment(&gpu.device, wgpu::TextureFormat::Rgba8Unorm);
    let depth = attachment(&gpu.device, wgpu::TextureFormat::Depth32Float);
    for material in [
        Material::from(PbrMaterial::new("PBR")),
        Material::from(BlinnPhongMaterial::new("Blinn-Phong")),
    ] {
        let mesh =
            Mesh::from_cpu_buffers("Plane", &primitive::plane(2, 2.0), Rc::new(material), &gpu);
        let mut renderer = RasterRenderer::new(&gpu, color.format());
        let mut draw = |lights: &[DirectionalLight]| {
            renderer.update_frame_bindings(
                &FrameBindingsData::new(&camera, lights, &[]),
                1.0,
                &gpu,
            );
            let image = render(&gpu, &mut renderer, &[&mesh], &color, &depth);
            let center: [u8; 4] = pixel(&image, 32).try_into().unwrap();
            center
        };
        assert_eq!(draw(&[]), [0, 0, 0, 255]);
        let first = draw(&lights[..1]);
        let second = draw(&lights[1..]);
        let combined = draw(&lights);
        for channel in 0..3 {
            assert!(first[channel] > 0 && second[channel] > 0);
            let sum = u16::from(first[channel]) + u16::from(second[channel]);
            assert!(sum < 255, "The additive comparison must not clip");
            assert!(u16::from(combined[channel]).abs_diff(sum) <= 1);
        }
        assert_eq!(draw(&lights[..1]), first);
        assert_eq!(draw(&[]), [0, 0, 0, 255]);
    }
}

#[test]
#[ignore = "GPU end-to-end check; run explicitly with approval"]
fn phong_point_lights_follow_inverse_square_and_scale_the_specular_lobe() {
    let gpu = pollster::block_on(WGPUContext::new(wgpu::Instance::default(), None));
    let camera = Camera::new(Orthographic {
        size_vertical: 4.0,
        near: 0.1,
        far: 10.0,
    });
    camera.transform_mut().position = DVec3::new(0.0, 3.0, 0.0);
    camera.transform_mut().look_at(DVec3::ZERO, DVec3::NEG_Z);
    let color = attachment(&gpu.device, wgpu::TextureFormat::Rgba8Unorm);
    let depth = attachment(&gpu.device, wgpu::TextureFormat::Depth32Float);
    for specular_only in [false, true] {
        let mut material = BlinnPhongMaterial::new("Photometric surface");
        if specular_only {
            material.set_diffuse_color(0.0, 0.0, 0.0);
            material.set_specular_color(1.0, 1.0, 1.0);
        } else {
            material.set_specular_color(0.0, 0.0, 0.0);
        }
        let mesh = Mesh::from_cpu_buffers(
            "Plane",
            &primitive::plane(2, 2.0),
            Rc::new(material.into()),
            &gpu,
        );
        let mut renderer = RasterRenderer::new(&gpu, color.format());
        let mut draw = |distance, intensity| {
            let mut light = crate::lights::PointLight::default();
            light.set_intensity(crate::photometry::Candela::new(intensity));
            // Keep incidence and reflection angles fixed while changing the distance.
            let texel_offset = 0.5 * camera.projection.size_vertical / f64::from(color.width());
            light.transform_mut().position = DVec3::new(texel_offset, distance, texel_offset);
            renderer.update_frame_bindings(
                &FrameBindingsData::new(&camera, &[], &[light]),
                1.0,
                &gpu,
            );
            let image = render(&gpu, &mut renderer, &[&mesh], &color, &depth);
            u16::from(pixel(&image, 32)[0])
        };
        let intensity = if specular_only { 0.1 } else { 1.0 };
        let near = draw(1.0, intensity);
        let far = draw(2.0, intensity);
        let dim = draw(1.0, intensity / 2.0);
        assert!(near > 30 && near < 250);
        assert!(near.abs_diff(4 * far) <= 4, "{near} != 4 * {far}");
        assert!(near.abs_diff(2 * dim) <= 2, "{near} != 2 * {dim}");
        assert_eq!(draw(1.0, 0.0), 0);
        if !specular_only {
            assert!(
                near.abs_diff(81) <= 1,
                "One candela at one meter on a white Lambertian surface must produce 1/pi nits"
            );
        }
    }
}

#[test]
#[ignore = "GPU end-to-end check; run explicitly with approval"]
fn exposure_scales_lighting_and_emission_in_fullscreen_pass() {
    use crate::blocks::{expose, rasterize};
    use crate::camera::Exposure;
    use crate::core::graph_executor::GraphExecutor;
    use crate::core::render_graph::RenderGraph;

    pollster::block_on(async {
        let mut executor = GraphExecutor::new().await;
        let camera = Camera::new(Orthographic {
            size_vertical: 4.0,
            near: 0.1,
            far: 10.0,
        });
        camera.transform_mut().position = DVec3::new(0.0, 3.0, 0.0);
        camera.transform_mut().look_at(DVec3::ZERO, DVec3::NEG_Z);
        let mut light = DirectionalLight::default();
        light.set_illuminance(Lux::new(50_000.0));
        let mut target = executor.create_target(32, 32);
        for emission_only in [false, true] {
            for mut material in [
                Material::from(PbrMaterial::new("PBR")),
                Material::from(BlinnPhongMaterial::new("Phong")),
            ] {
                if emission_only {
                    match &mut material {
                        Material::Pbr(pbr) => pbr.set_emissive_luminance(Nits::new(5_000.0)),
                        Material::BlinnPhong(phong) => {
                            phong.set_ambient_luminance(Nits::new(5_000.0));
                        }
                    }
                }
                let mesh = Mesh::from_cpu_buffers(
                    "Plane",
                    &primitive::plane(2, 2.0),
                    Rc::new(material),
                    executor.wgpu_context(),
                );
                let mut samples = Vec::new();
                for exposure in [
                    Exposure::default(),
                    Exposure::default().with_compensation_stops(1.0),
                    Exposure::default(),
                ] {
                    let lights = if emission_only {
                        &[][..]
                    } else {
                        std::slice::from_ref(&light)
                    };
                    let mut graph = RenderGraph::new();
                    let scene = rasterize(
                        FrameBindingsData::new(&camera, lights, &[]),
                        [&mesh],
                        wgpu::TextureFormat::Rgba32Float,
                        [0.1, 0.1, 0.2, 1.0],
                        &mut graph,
                    );
                    let image = expose(
                        scene.color,
                        exposure,
                        wgpu::TextureFormat::Rgba8Unorm,
                        &mut graph,
                    );
                    graph.export(image);
                    executor.execute(graph, &mut target).unwrap();
                    let image = target.read_image(executor.wgpu_context()).await.unwrap();
                    samples.push(u16::from(image.get_pixel(16, 16)[0]));
                }
                let [baseline, brighter, restored] = samples[..] else {
                    unreachable!()
                };
                assert!(baseline > 5 && baseline < 120);
                assert!(brighter.abs_diff(2 * baseline) <= 1);
                assert_eq!(restored, baseline);
            }
        }
    });
}

#[test]
#[ignore = "GPU end-to-end check; run explicitly with approval"]
fn bright_emission_survives_float32_rasterization_and_fullscreen_exposure() {
    use crate::blocks::{expose, rasterize, tone_map};
    use crate::core::graph_executor::GraphExecutor;
    use crate::core::render_graph::RenderGraph;

    pollster::block_on(async {
        let mut executor = GraphExecutor::new().await;
        let camera = Camera::new(Orthographic {
            size_vertical: 4.0,
            near: 0.1,
            far: 10.0,
        });
        camera.transform_mut().position = DVec3::new(0.0, 3.0, 0.0);
        camera.transform_mut().look_at(DVec3::ZERO, DVec3::NEG_Z);
        for material in [
            {
                let mut pbr = PbrMaterial::new("Bright emitter");
                pbr.set_emissive_luminance(Nits::new(100_000_000.0));
                Material::from(pbr)
            },
            {
                let mut phong = BlinnPhongMaterial::new("Bright ambient");
                phong.set_ambient_luminance(Nits::new(100_000_000.0));
                Material::from(phong)
            },
        ] {
            let mesh = Mesh::from_cpu_buffers(
                "Plane",
                &primitive::plane(2, 2.0),
                Rc::new(material),
                executor.wgpu_context(),
            );
            let mut graph = RenderGraph::new();
            let scene = rasterize(
                FrameBindingsData::new(&camera, &[], &[]),
                [&mesh],
                wgpu::TextureFormat::Rgba32Float,
                [0.1, 0.1, 0.2, 1.0],
                &mut graph,
            );
            let exposed = expose(
                scene.color,
                camera.exposure,
                wgpu::TextureFormat::Rgba16Float,
                &mut graph,
            );
            let image = tone_map(exposed, wgpu::TextureFormat::Rgba8UnormSrgb, &mut graph);
            graph.export(image);
            let mut target = executor.create_target(32, 32);
            executor.execute(graph, &mut target).unwrap();
            let image = target.read_image(executor.wgpu_context()).await.unwrap();
            let pixel = image.get_pixel(16, 16);
            assert!(
                pixel.0[..3].iter().all(|&channel| channel >= 254),
                "Bright emission must stay finite through HDR and tone mapping: {pixel:?}"
            );
            assert_eq!(pixel[3], 255);
        }
    });
}

fn set_unlit_color(material: &mut Material, color: [f32; 3]) {
    match material {
        Material::Pbr(material) => {
            material.emissive_color = color;
            material.emissive_luminance = Nits::new(1.0);
        }
        Material::BlinnPhong(material) => {
            material.ambient_color = color;
            material.ambient_luminance = Nits::new(1.0);
        }
    }
}

fn attachment(device: &wgpu::Device, format: wgpu::TextureFormat) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Material Runtime Test"),
        size: wgpu::Extent3d {
            width: 64,
            height: 64,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    })
}

fn render(
    gpu: &WGPUContext,
    renderer: &mut RasterRenderer,
    meshes: &[&Mesh],
    color: &wgpu::Texture,
    depth: &wgpu::Texture,
) -> Vec<u8> {
    let mut encoder = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    renderer.encode(
        meshes.iter().copied(),
        gpu,
        &mut encoder,
        &RenderTarget {
            color: &color.create_view(&wgpu::TextureViewDescriptor::default()),
            depth: &depth.create_view(&wgpu::TextureViewDescriptor::default()),
        },
        [0.1, 0.1, 0.2, 1.0],
    );
    gpu.queue.submit([encoder.finish()]);
    pollster::block_on(crate::core::readback::read_rgba8(
        &gpu.device,
        &gpu.queue,
        color,
    ))
    .expect("Failed to read the rendered test texture")
    .into_raw()
}

fn pixel(image: &[u8], x: usize) -> &[u8] {
    &image[(32 * 64 + x) * 4..][..4]
}

fn assert_channel(image: &[u8], x: usize, channel: usize) {
    let pixel = pixel(image, x);
    assert!(pixel[channel] > 100, "x={x}: {pixel:?}");
    for (index, &value) in pixel[..3].iter().enumerate() {
        if index != channel {
            assert_eq!(value, 0, "x={x}: {pixel:?}");
        }
    }
}
