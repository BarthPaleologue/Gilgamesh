use crate::core::fullscreen_pass;
use crate::core::render_graph::{Image, ImageDescriptor, ImageSize, RenderGraph};
use wgpu::{TextureSampleType, TextureUsages};

/// Resizes a float-sampled color image using nearest-neighbor sampling.
/// Preserves its channels, format and color space, including HDR values.
/// The source format must support render attachments.
/// `size` is resolved relative to the frame target, as for other graph images.
pub fn resize_nearest(source: Image, size: ImageSize, graph: &mut RenderGraph<'_>) -> Image {
    let descriptor = graph.image_descriptor(source);
    assert!(
        matches!(
            descriptor.format.sample_type(None, None),
            Some(TextureSampleType::Float { .. })
        ),
        "resize_nearest requires a float-sampled color format, got {:?}",
        descriptor.format
    );
    let output = graph.create_image("Nearest Resize", ImageDescriptor { size, ..descriptor });
    graph.add_pass(
        "Nearest Resize",
        &[(source, TextureUsages::TEXTURE_BINDING)],
        &[(output, TextureUsages::RENDER_ATTACHMENT)],
        move |context| {
            let shader = include_str!("resize_nearest.wgsl");
            let (pipeline, parameters) =
                context
                    .cache
                    .get_or_insert_with((shader, descriptor.format), || {
                        (
                            fullscreen_pass::create_pipeline(
                                &context.gpu.device,
                                shader,
                                "fs_main",
                                descriptor.format,
                            ),
                            context.gpu.device.create_buffer(&wgpu::BufferDescriptor {
                                label: Some("Resize Dimensions"),
                                size: 16,
                                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                                mapped_at_creation: false,
                            }),
                        )
                    });
            let dimensions = context.images.get(output).texture().size();
            context.gpu.queue.write_buffer(
                parameters,
                0,
                bytemuck::cast_slice(&[dimensions.width, dimensions.height, 0, 0]),
            );
            let bindings = context
                .gpu
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Nearest Resize"),
                    layout: &pipeline.get_bind_group_layout(0),
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(
                                context.images.get(source),
                            ),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
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
