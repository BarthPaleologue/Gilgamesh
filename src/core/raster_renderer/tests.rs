use super::*;
use crate::camera::{Camera, Orthographic};
use crate::geometry::primitive;
use crate::lights::directional_light::DirectionalLight;
use crate::lights::light::Light;
use crate::materials::blinn_phong::BlinnPhongMaterial;
use crate::materials::pbr::PbrMaterial;
use glam::DVec3;

#[test]
#[ignore = "GPU end-to-end check; run explicitly with approval"]
fn shared_materials_have_renderer_local_runtime() {
    let gpu = pollster::block_on(WGPUContext::new(wgpu::Instance::default(), None));
    let camera = Camera::new(Orthographic {
        size_vertical: 4.0,
        aspect_ratio: 1.0,
        near: 0.1,
        far: 10.0,
    });
    camera.transform_mut().position = DVec3::new(0.0, 3.0, 0.0);
    camera.transform_mut().look_at(DVec3::ZERO, DVec3::NEG_Z);
    let mut light = DirectionalLight::default();
    light.set_intensity(0.0);
    let depth = attachment(&gpu.device, wgpu::TextureFormat::Depth32Float);

    for mut description in [
        Material::from(PbrMaterial::new("PBR")),
        Material::from(BlinnPhongMaterial::new("Blinn-Phong")),
    ] {
        set_unlit_color(&mut description, [1.0, 0.0, 0.0]);
        let material = Rc::new(description);
        let source = Rc::downgrade(&material);
        let mut left =
            Mesh::from_cpu_buffers("Left", primitive::plane(2, 1.0), material.clone(), &gpu);
        let right =
            Mesh::from_cpu_buffers("Right", primitive::plane(2, 1.0), material.clone(), &gpu);
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
            renderer.update_frame_bindings(FrameBindingsData::new(&camera, &light, &[]), &gpu);
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
        let mesh = Mesh::from_cpu_buffers("Edited", primitive::plane(2, 1.0), material, &gpu);
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

fn set_unlit_color(material: &mut Material, color: [f32; 3]) {
    match material {
        Material::Pbr(material) => material.emissive_color = color,
        Material::BlinnPhong(material) => material.ambient_color = color,
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
    let mut encoder = gpu.device.create_command_encoder(&Default::default());
    renderer.encode(
        meshes.iter().copied(),
        gpu,
        &mut encoder,
        RenderTarget {
            color: &color.create_view(&Default::default()),
            depth: &depth.create_view(&Default::default()),
        },
    );
    gpu.queue.submit([encoder.finish()]);
    pollster::block_on(crate::core::readback::read_rgba8(
        &gpu.device,
        &gpu.queue,
        color,
    ))
    .unwrap()
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
