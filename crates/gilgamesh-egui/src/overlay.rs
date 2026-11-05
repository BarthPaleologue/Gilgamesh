use crate::Egui;
use crate::renderer::GpuRenderer;
use egui_wgpu::ScreenDescriptor;
use gilgamesh::core::render_graph::{Image, RenderGraph};
use wgpu::TextureUsages;

/// Draws the last prepared UI over an existing image after scene tone mapping.
/// Returns a successor version backed by the same texture, preserving pixels outside the UI.
/// egui-wgpu handles the target's color space directly.
///
/// # Panics
/// Panics if the image belongs to another graph.
pub fn egui_overlay<'a>(ui: &'a Egui, background: Image, graph: &mut RenderGraph<'a>) -> Image {
    let format = graph.image_descriptor(background).format;
    let output = graph.next_version(background);
    graph.add_pass(
        "egui paint",
        &[],
        &[(output, TextureUsages::RENDER_ATTACHMENT)],
        move |context| {
            let view = context.images.get(output);
            let screen = ScreenDescriptor {
                size_in_pixels: [view.texture().width(), view.texture().height()],
                pixels_per_point: ui.pixels_per_point,
            };
            let mut prepared = ui.renderer.borrow_mut();
            if prepared
                .as_ref()
                .is_none_or(|state| state.format != format || state.device != context.gpu.device)
            {
                *prepared = Some(GpuRenderer::new(context.gpu, format));
            }
            let state = prepared.as_mut().expect("egui renderer was initialized");
            state.sync_textures(context.gpu, &ui.textures, ui.generation);
            let renderer = &mut state.renderer;
            let commands = renderer.update_buffers(
                &context.gpu.device,
                &context.gpu.queue,
                context.encoder,
                &ui.paint_jobs,
                &screen,
            );
            context.gpu.queue.submit(commands);
            let mut pass = context
                .encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("egui"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    ..Default::default()
                })
                .forget_lifetime();
            renderer.render(&mut pass, &ui.paint_jobs, &screen);
        },
    );
    output
}
