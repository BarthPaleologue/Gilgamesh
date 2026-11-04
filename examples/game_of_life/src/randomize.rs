use gilgamesh::core::render_graph::{Image, RenderGraph, TextureUsages};

pub fn randomize(target: Image, seed: u32, graph: &mut RenderGraph<'_>) -> Image {
    let target = graph.next_version(target);
    graph.add_pass(
        "Randomize Game of Life",
        &[],
        &[(target, TextureUsages::STORAGE_BINDING)],
        move |context| {
            let shader = include_str!("randomize.wgsl");
            let (pipeline, parameters) = context.cache.get_or_insert_with(shader, || {
                let module =
                    context
                        .gpu
                        .device
                        .create_shader_module(wgpu::ShaderModuleDescriptor {
                            label: Some("Randomize Game of Life"),
                            source: wgpu::ShaderSource::Wgsl(shader.into()),
                        });
                (
                    context
                        .gpu
                        .device
                        .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                            label: Some("Randomize Game of Life"),
                            layout: None,
                            module: &module,
                            entry_point: Some("main"),
                            compilation_options: Default::default(),
                            cache: None,
                        }),
                    context.gpu.device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some("Random Seed"),
                        size: 16,
                        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                        mapped_at_creation: false,
                    }),
                )
            });
            let mut bytes = [0; 16];
            bytes[..4].copy_from_slice(&seed.to_ne_bytes());
            context.gpu.queue.write_buffer(parameters, 0, &bytes);
            let bindings = context
                .gpu
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Randomize Game of Life"),
                    layout: &pipeline.get_bind_group_layout(0),
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(
                                context.images.get(target),
                            ),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: parameters.as_entire_binding(),
                        },
                    ],
                });
            let size = context.images.get(target).texture().size();
            let mut pass = context.encoder.begin_compute_pass(&Default::default());
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, &bindings, &[]);
            pass.dispatch_workgroups(size.width.div_ceil(8), size.height.div_ceil(8), 1);
        },
    );
    target
}
