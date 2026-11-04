use crate::core::fullscreen_pass;
use crate::core::render_graph::{Image, RenderGraph};
use wgpu::{TextureSampleType, TextureUsages};

/// Combines color images across a vertical divider, preserving the left image's size and format.
/// `split` is a fraction in [0, 1] measured from the left. The right image is resampled
/// with nearest-neighbor lookup when its dimensions differ. HDR stays HDR.
pub fn vertical_split_screen(
    left: Image,
    right: Image,
    split: f32,
    graph: &mut RenderGraph<'_>,
) -> Image {
    assert!(
        (0.0..=1.0).contains(&split),
        "Split position must be in [0, 1]"
    );
    let descriptor = graph.image_descriptor(left);
    for image in [left, right] {
        assert!(
            matches!(
                graph.image_descriptor(image).format.sample_type(None, None),
                Some(TextureSampleType::Float { .. })
            ),
            "vertical_split_screen requires float-sampled color images"
        );
    }
    let output = graph.create_image("Vertical Split Screen", descriptor);
    graph.add_pass(
        "Vertical Split Screen",
        &[
            (left, TextureUsages::TEXTURE_BINDING),
            (right, TextureUsages::TEXTURE_BINDING),
        ],
        &[(output, TextureUsages::RENDER_ATTACHMENT)],
        move |context| {
            let shader = include_str!("vertical_split_screen.wgsl");
            let (pipeline, parameters) =
                context.cache.get_or_insert_with(descriptor.format, || {
                    (
                        fullscreen_pass::create_pipeline(
                            &context.gpu.device,
                            shader,
                            "fs_main",
                            descriptor.format,
                        ),
                        context.gpu.device.create_buffer(&wgpu::BufferDescriptor {
                            label: Some("Split Position"),
                            size: 16,
                            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                            mapped_at_creation: false,
                        }),
                    )
                });
            context
                .gpu
                .queue
                .write_buffer(parameters, 0, &split.to_ne_bytes());
            let bindings = context
                .gpu
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Vertical Split Screen"),
                    layout: &pipeline.get_bind_group_layout(0),
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(context.images.get(left)),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(context.images.get(right)),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: parameters.as_entire_binding(),
                        },
                    ],
                });
            fullscreen_pass::encode(
                context.encoder,
                pipeline,
                &bindings,
                context.images.get(output),
            );
        },
    );
    output
}
