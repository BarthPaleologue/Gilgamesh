use crate::core::render_graph::{
    GraphError, GraphPlan, PassCache, PassContext, PassImages, RenderGraph,
};
use crate::core::render_target::create_color_view;
use crate::core::wgpu_context::WGPUContext;

/// Executes borrowed render descriptions and retains their GPU state between renders.
#[must_use]
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

    #[must_use]
    pub fn wgpu_context(&self) -> &WGPUContext {
        &self.wgpu_context
    }

    /// # Panics
    /// Panics if either dimension is zero.
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
    ///
    /// # Errors
    /// Returns an error for invalid graph dependencies, unsupported image usages,
    /// or image dimensions incompatible with the device or frame target.
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
        let keys = self.image_keys(&graph, &plan, width, height)?;
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
        let transient = plan_transients(&graph, &plan, &keys);
        for key in transient.keys {
            let pooled = if let Some(index) = available.iter().position(|image| image.key == key) {
                available.swap_remove(index)
            } else {
                PooledImage {
                    key,
                    view: create_image(&self.wgpu_context.device, "Transient Image", key),
                }
            };
            self.images.push(pooled);
        }
        let mut views: Vec<wgpu::TextureView> = Vec::with_capacity(graph.images.len());
        for (index, image) in graph.images.iter().enumerate() {
            let view = if image.allocation != index {
                views[image.allocation].clone()
            } else if image.allocation == graph.images[plan.output.index].allocation {
                target.output().clone()
            } else if let Some(imported) = image.imported {
                imported.create_view(&wgpu::TextureViewDescriptor::default())
            } else {
                self.images[transient.slots[index].expect("Created image needs a transient slot")]
                    .view
                    .clone()
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

    fn image_keys(
        &self,
        graph: &RenderGraph<'_>,
        plan: &GraphPlan,
        width: u32,
        height: u32,
    ) -> Result<Vec<ImageKey>, GraphError> {
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
        Ok(keys)
    }
}

struct TransientPlan {
    keys: Vec<ImageKey>,
    slots: Vec<Option<usize>>,
}

fn plan_transients(graph: &RenderGraph<'_>, plan: &GraphPlan, keys: &[ImageKey]) -> TransientPlan {
    let mut roots: Vec<_> = graph
        .images
        .iter()
        .enumerate()
        .filter_map(|(index, image)| {
            (image.allocation == index
                && image.imported.is_none()
                && index != graph.images[plan.output.index].allocation)
                .then_some(index)
        })
        .collect();
    roots.sort_by_key(|&index| plan.lifetimes[index].expect("Created image has a writer").0);
    let mut allocations: Vec<(ImageKey, usize)> = Vec::new();
    let mut slots = vec![None; graph.images.len()];
    for image in roots {
        let key = keys[image];
        let (first, last) = plan.lifetimes[image].unwrap();
        let slot = allocations
            .iter()
            .position(|(existing, last_use)| {
                *last_use < first
                    && existing.width == key.width
                    && existing.height == key.height
                    && existing.format == key.format
            })
            .unwrap_or_else(|| {
                allocations.push((key, last));
                allocations.len() - 1
            });
        let (slot_key, last_use) = &mut allocations[slot];
        slot_key.usage |= key.usage;
        *last_use = last;
        slots[image] = Some(slot);
    }
    TransientPlan {
        keys: allocations.into_iter().map(|(key, _)| key).collect(),
        slots,
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
        .create_view(&wgpu::TextureViewDescriptor::default())
}

/// Owns the exported image independently of the executor's intermediate texture pool.
#[must_use]
pub struct FrameTarget {
    image: wgpu::TextureView,
}

impl FrameTarget {
    #[must_use]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::render_graph::{Image, ImageDescriptor, ImageSize};
    use wgpu::{TextureFormat as Format, TextureUsages as Usage};

    fn image(format: Format, size: ImageSize, graph: &mut RenderGraph<'_>) -> Image {
        graph.create_image("Test image", ImageDescriptor { size, format })
    }

    fn transient_plan(graph: &RenderGraph<'_>) -> TransientPlan {
        let plan = graph.compile().unwrap();
        let keys: Vec<_> = graph
            .images
            .iter()
            .zip(&plan.usages)
            .map(|(image, &usage)| {
                let (width, height) = image.descriptor.size.resolve(4, 4).unwrap();
                ImageKey {
                    width,
                    height,
                    format: image.descriptor.format,
                    usage,
                }
            })
            .collect();
        plan_transients(graph, &plan, &keys)
    }

    #[test]
    fn linear_chain_uses_two_transient_slots_and_reserves_export() {
        let mut graph = RenderGraph::new();
        let images: Vec<_> = (0..9)
            .map(|_| image(Format::Rgba8Unorm, ImageSize::Target, &mut graph))
            .collect();
        graph.add_pass(
            "Initialize",
            &[],
            &[(images[0], Usage::RENDER_ATTACHMENT)],
            |_| {},
        );
        for pair in images.windows(2) {
            graph.add_pass(
                "Copy",
                &[(pair[0], Usage::COPY_SRC)],
                &[(pair[1], Usage::COPY_DST)],
                |_| {},
            );
        }
        graph.export(images[8]);
        let plan = transient_plan(&graph);
        assert_eq!(plan.keys.len(), 2);
        for (index, &image) in images[..8].iter().enumerate() {
            assert_eq!(plan.slots[image.index], Some(index % 2));
        }
        assert_eq!(plan.slots[images[8].index], None);
        assert!(
            plan.keys[0]
                .usage
                .contains(Usage::RENDER_ATTACHMENT | Usage::COPY_DST | Usage::COPY_SRC)
        );
    }

    #[test]
    fn a_live_branch_keeps_its_slot_until_the_merge() {
        let mut graph = RenderGraph::new();
        let images: Vec<_> = (0..7)
            .map(|_| image(Format::Rgba8Unorm, ImageSize::Target, &mut graph))
            .collect();
        graph.add_pass(
            "Initialize",
            &[],
            &[(images[0], Usage::RENDER_ATTACHMENT)],
            |_| {},
        );
        for pair in images[..6].windows(2) {
            graph.add_pass(
                "Filter",
                &[(pair[0], Usage::TEXTURE_BINDING)],
                &[(pair[1], Usage::RENDER_ATTACHMENT)],
                |_| {},
            );
        }
        graph.add_pass(
            "Merge",
            &[
                (images[0], Usage::TEXTURE_BINDING),
                (images[5], Usage::TEXTURE_BINDING),
            ],
            &[(images[6], Usage::RENDER_ATTACHMENT)],
            |_| {},
        );
        graph.export(images[6]);
        let plan = transient_plan(&graph);
        assert_eq!(plan.keys.len(), 3);
        for &image in &images[1..6] {
            assert_ne!(plan.slots[images[0].index], plan.slots[image.index]);
        }
    }

    #[test]
    fn incompatible_formats_and_sizes_use_separate_slots() {
        let mut graph = RenderGraph::new();
        let a = image(Format::Rgba8Unorm, ImageSize::Target, &mut graph);
        let b = image(Format::Rgba16Float, ImageSize::Target, &mut graph);
        let c = image(
            Format::Rgba8Unorm,
            ImageSize::Fixed {
                width: 2,
                height: 2,
            },
            &mut graph,
        );
        let d = image(Format::Rgba8Unorm, ImageSize::Target, &mut graph);
        let output = image(Format::Rgba8Unorm, ImageSize::Target, &mut graph);
        graph.add_pass("Initialize", &[], &[(a, Usage::RENDER_ATTACHMENT)], |_| {});
        for pair in [a, b, c, d, output].windows(2) {
            graph.add_pass(
                "Resize",
                &[(pair[0], Usage::TEXTURE_BINDING)],
                &[(pair[1], Usage::RENDER_ATTACHMENT)],
                |_| {},
            );
        }
        graph.export(output);
        let plan = transient_plan(&graph);
        assert_eq!(plan.keys.len(), 3);
        assert_eq!(plan.slots[a.index], plan.slots[d.index]);
        assert_ne!(plan.slots[a.index], plan.slots[b.index]);
        assert_ne!(plan.slots[a.index], plan.slots[c.index]);
    }
}
