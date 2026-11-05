pub(crate) fn create_pipeline(
    device: &wgpu::Device,
    fragment: &str,
    entry_point: &str,
    formats: &[wgpu::TextureFormat],
) -> wgpu::RenderPipeline {
    let source = [include_str!("fullscreen_pass.wgsl"), fragment].join("\n");
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(entry_point),
        source: wgpu::ShaderSource::Wgsl(source.into()),
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(entry_point),
        layout: None,
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some(entry_point),
            targets: &formats
                .iter()
                .map(|&format| {
                    Some(wgpu::ColorTargetState {
                        format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })
                })
                .collect::<Vec<_>>(),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    })
}

pub(crate) fn encode(
    encoder: &mut wgpu::CommandEncoder,
    pipeline: &wgpu::RenderPipeline,
    bindings: &wgpu::BindGroup,
    target: &wgpu::TextureView,
) {
    encode_targets(encoder, pipeline, Some(bindings), &[target]);
}

pub(crate) fn encode_targets(
    encoder: &mut wgpu::CommandEncoder,
    pipeline: &wgpu::RenderPipeline,
    bindings: Option<&wgpu::BindGroup>,
    targets: &[&wgpu::TextureView],
) {
    let attachments: Vec<_> = targets
        .iter()
        .map(|&view| {
            Some(wgpu::RenderPassColorAttachment {
                view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })
        })
        .collect();
    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        color_attachments: &attachments,
        ..Default::default()
    });
    pass.set_pipeline(pipeline);
    if let Some(bindings) = bindings {
        pass.set_bind_group(0, bindings, &[]);
    }
    pass.draw(0..3, 0..1);
}
