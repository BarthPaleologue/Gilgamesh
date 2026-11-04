use gilgamesh::core::render_graph::{Image, RenderGraph, TextureUsages};

pub fn evolve(
    previous: Image,
    current: Image,
    step_due: bool,
    graph: &mut RenderGraph<'_>,
) -> Image {
    let current = graph.next_version(current);
    graph.add_pass(
        "Evolve Game of Life",
        &[(previous, TextureUsages::TEXTURE_BINDING)],
        &[(current, TextureUsages::STORAGE_BINDING)],
        move |context| {
            // Keep this pass and its pipeline cached between simulation steps.
            let pipeline = context.cache.get_or_insert_with((), || {
                let shader =
                    context
                        .gpu
                        .device
                        .create_shader_module(wgpu::ShaderModuleDescriptor {
                            label: Some("Game of Life"),
                            source: wgpu::ShaderSource::Wgsl(include_str!("evolve.wgsl").into()),
                        });
                context
                    .gpu
                    .device
                    .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                        label: Some("Game of Life"),
                        layout: None,
                        module: &shader,
                        entry_point: Some("main"),
                        compilation_options: Default::default(),
                        cache: None,
                    })
            });
            if !step_due {
                return;
            }
            let bindings = context
                .gpu
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Game of Life"),
                    layout: &pipeline.get_bind_group_layout(0),
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(
                                context.images.get(previous),
                            ),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(
                                context.images.get(current),
                            ),
                        },
                    ],
                });
            let size = context.images.get(current).texture().size();
            let mut pass = context.encoder.begin_compute_pass(&Default::default());
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, &bindings, &[]);
            pass.dispatch_workgroups(size.width.div_ceil(8), size.height.div_ceil(8), 1);
        },
    );
    current
}
