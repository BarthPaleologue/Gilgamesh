use crate::core::color_pass::ColorPass;
use crate::core::render_graph::{Image, ImageDescriptor, RenderGraph};
use wgpu::{TextureFormat, TextureUsages};

/// Applies Khronos PBR Neutral to linear HDR. Choose an sRGB output format for PNG captures.
pub fn tone_map(source: Image, format: TextureFormat, graph: &mut RenderGraph<'_>) -> Image {
    let output = graph.create_image(
        "Tone Mapped Image",
        ImageDescriptor {
            format,
            ..graph.image_descriptor(source)
        },
    );
    graph.add_pass(
        "Tone Mapping",
        &[(source, TextureUsages::TEXTURE_BINDING)],
        &[(output, TextureUsages::RENDER_ATTACHMENT)],
        move |context| {
            let pass = context.cache.get_or_insert_with(format, || {
                ColorPass::tone_mapping(&context.gpu.device, format)
            });
            let source = pass.bind_source(&context.gpu.device, context.images.get(source));
            pass.encode(context.encoder, &source, context.images.get(output));
        },
    );
    output
}
