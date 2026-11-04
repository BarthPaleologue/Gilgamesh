use crate::core::render_graph::{GraphError, PassCache, PassContext, PassImages, RenderGraph};
use crate::core::render_target::create_color_view;
use crate::core::wgpu_context::WGPUContext;

/// Executes borrowed render descriptions and retains their GPU state between renders.
pub struct GraphExecutor {
    wgpu_context: WGPUContext,
    images: Vec<PooledImage>,
    passes: Vec<PassCache>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ImageKey {
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
    usage: wgpu::TextureUsages,
}

struct PooledImage {
    key: ImageKey,
    view: wgpu::TextureView,
}

impl GraphExecutor {
    pub async fn new() -> Self {
        let instance = wgpu::util::new_instance_with_webgpu_detection(
            wgpu::InstanceDescriptor::new_without_display_handle(),
        )
        .await;
        Self::from_context(WGPUContext::new(instance, None).await)
    }

    pub fn from_context(wgpu_context: WGPUContext) -> Self {
        Self {
            wgpu_context,
            images: Vec::new(),
            passes: Vec::new(),
        }
    }

    pub fn wgpu_context(&self) -> &WGPUContext {
        &self.wgpu_context
    }

    pub fn create_target(&self, width: u32, height: u32) -> FrameTarget {
        assert!(
            width > 0 && height > 0,
            "Frame target dimensions must be nonzero"
        );
        FrameTarget {
            image: create_color_view(
                &self.wgpu_context.device,
                "Frame Image",
                wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                wgpu::TextureFormat::Rgba8UnormSrgb,
            ),
        }
    }

    /// Returns whether the output image was replaced, invalidating its bindings.
    pub fn resize_target(&self, target: &mut FrameTarget, width: u32, height: u32) -> bool {
        let texture = target.output().texture();
        if (texture.width(), texture.height()) == (width, height) {
            return false;
        }
        *target = self.create_target(width, height);
        true
    }

    /// Validates dependencies and image descriptors, then encodes and submits the graph.
    pub fn execute(
        &mut self,
        graph: RenderGraph<'_>,
        target: &mut FrameTarget,
    ) -> Result<(), GraphError> {
        let mut encoder =
            self.wgpu_context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Graph Encoder"),
                });
        self.encode(graph, target, &mut encoder)?;
        self.wgpu_context.queue.submit([encoder.finish()]);
        Ok(())
    }

    pub(crate) fn encode(
        &mut self,
        graph: RenderGraph<'_>,
        target: &mut FrameTarget,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Result<(), GraphError> {
        let plan = graph.compile()?;
        let output = target.output().texture();
        let (width, height) = (output.width(), output.height());
        let limits = self.wgpu_context.device.limits();
        let mut keys = Vec::with_capacity(graph.images.len());
        for (image, &usage) in graph.images.iter().zip(&plan.usages) {
            let (width, height) = image.descriptor.size.resolve(width, height)?;
            if width > limits.max_texture_dimension_2d || height > limits.max_texture_dimension_2d {
                return Err(GraphError::new(format!(
                    "{}: image exceeds device dimensions",
                    image.label
                )));
            }
            let features = image
                .descriptor
                .format
                .guaranteed_format_features(self.wgpu_context.device.features());
            if !features.allowed_usages.contains(usage) {
                return Err(GraphError::new(format!(
                    "{}: {:?} does not support usages {usage:?}",
                    image.label, image.descriptor.format
                )));
            }
            keys.push(ImageKey {
                width,
                height,
                format: image.descriptor.format,
                usage,
            });
        }
        let output_key = keys[plan.output.index];
        if (output_key.width, output_key.height) != (width, height) {
            return Err(GraphError::new(
                "Exported image dimensions must match the frame target",
            ));
        }
        if target.output().texture().format() != output_key.format
            || !target.output().texture().usage().contains(output_key.usage)
            || graph
                .images
                .iter()
                .any(|image| image.imported == Some(target.output().texture()))
        {
            target.image = create_image(&self.wgpu_context.device, "Frame Image", output_key);
        }
        let mut available = std::mem::take(&mut self.images);
        available.retain(|pooled| {
            !graph
                .images
                .iter()
                .any(|image| image.imported == Some(pooled.view.texture()))
        });
        let mut views: Vec<wgpu::TextureView> = Vec::with_capacity(graph.images.len());
        for (index, (image, key)) in graph.images.iter().zip(keys).enumerate() {
            if image.allocation != index {
                views.push(views[image.allocation].clone());
                continue;
            }
            let view = if image.allocation == graph.images[plan.output.index].allocation {
                target.output().clone()
            } else if let Some(imported) = image.imported {
                imported.create_view(&Default::default())
            } else {
                let pooled =
                    if let Some(index) = available.iter().position(|image| image.key == key) {
                        available.swap_remove(index)
                    } else {
                        PooledImage {
                            key,
                            view: create_image(&self.wgpu_context.device, &image.label, key),
                        }
                    };
                let view = pooled.view.clone();
                self.images.push(pooled);
                view
            };
            views.push(view);
        }
        // Retain only the last execution's working set, including when the graph or size changes.
        self.passes
            .resize_with(graph.passes.len(), PassCache::default);
        let mut passes: Vec<_> = graph.passes.into_iter().map(Some).collect();
        for index in plan.order {
            let pass = passes[index].take().unwrap();
            let cache = &mut self.passes[index];
            cache.begin_render();
            encoder.push_debug_group(&pass.label);
            (pass.encode)(&mut PassContext {
                gpu: &self.wgpu_context,
                encoder,
                images: PassImages {
                    graph: graph.id,
                    views: &views,
                    reads: &pass.reads,
                    writes: &pass.writes,
                },
                cache,
            });
            encoder.pop_debug_group();
            cache.end_render();
        }
        Ok(())
    }
}

fn create_image(device: &wgpu::Device, label: &str, key: ImageKey) -> wgpu::TextureView {
    device
        .create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width: key.width,
                height: key.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: key.format,
            usage: key.usage,
            view_formats: &[],
        })
        .create_view(&Default::default())
}

/// Owns the exported image independently of the executor's intermediate texture pool.
pub struct FrameTarget {
    image: wgpu::TextureView,
}

impl FrameTarget {
    pub fn output(&self) -> &wgpu::TextureView {
        &self.image
    }

    /// Starts a snapshot immediately. The future owns its readback resources.
    /// Browser readback requires WebGPU; WebGL returns `Unsupported`.
    pub fn read_image(
        &self,
        ctx: &WGPUContext,
    ) -> impl std::future::Future<Output = std::io::Result<image::RgbaImage>> + use<> {
        crate::core::readback::read_rgba8(&ctx.device, &ctx.queue, self.output().texture())
    }
}
