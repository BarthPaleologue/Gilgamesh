use crate::core::fullscreen_pass;
use crate::core::render_graph::{Image, ImageDescriptor, RenderGraph};
use wgpu::{TextureFormat, TextureSampleType, TextureUsages};

/// Displays raw depth as grayscale in an sRGB image, without linearization.
pub fn visualize_depth(source: Image, graph: &mut RenderGraph<'_>) -> Image {
    let descriptor = graph.image_descriptor(source);
    assert_eq!(
        descriptor.format.sample_type(None, None),
        Some(TextureSampleType::Depth),
        "visualize_depth requires a depth format, got {:?}",
        descriptor.format
    );
    let output = graph.create_image(
        "Visualize Depth",
        ImageDescriptor {
            format: TextureFormat::Rgba8UnormSrgb,
            ..descriptor
        },
    );
    graph.add_pass(
        "Visualize Depth",
        &[(source, TextureUsages::TEXTURE_BINDING)],
        &[(output, TextureUsages::RENDER_ATTACHMENT)],
        move |context| {
            let (pipeline, sampler) = context.cache.get_or_insert_with((), || {
                (
                    fullscreen_pass::create_pipeline(
                        &context.gpu.device,
                        include_str!("visualize_depth.wgsl"),
                        "fs_main",
                        TextureFormat::Rgba8UnormSrgb,
                    ),
                    context
                        .gpu
                        .device
                        .create_sampler(&wgpu::SamplerDescriptor::default()),
                )
            });
            let bindings = context
                .gpu
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Visualize Depth"),
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
                            resource: wgpu::BindingResource::Sampler(sampler),
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
