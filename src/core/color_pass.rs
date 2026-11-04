use crate::core::fullscreen_pass;

/// A fullscreen fragment pass over equally sized, single-sample textures.
pub struct ColorPass {
    pipeline: wgpu::RenderPipeline,
    source_layout: wgpu::BindGroupLayout,
    entry_point: &'static str,
}

impl ColorPass {
    /// Maps linear Rec. 709 HDR colors to linear SDR using Khronos PBR Neutral.
    /// An sRGB destination performs the output encoding automatically.
    pub fn tone_mapping(device: &wgpu::Device, target_format: wgpu::TextureFormat) -> Self {
        Self::new(
            device,
            target_format,
            include_str!("../blocks/tone_map.wgsl"),
            "fs_main",
        )
    }

    /// Transfers an SDR image, encoding sRGB explicitly for non-sRGB destinations.
    /// The source must be sampled as linear color, including through an sRGB view.
    pub fn image_blit(device: &wgpu::Device, target_format: wgpu::TextureFormat) -> Self {
        let entry_point = if target_format.is_srgb() {
            "image_blit"
        } else {
            "encode_srgb"
        };
        Self::new(
            device,
            target_format,
            include_str!("image_blit.wgsl"),
            entry_point,
        )
    }

    pub fn bind_source(
        &self,
        device: &wgpu::Device,
        source: &wgpu::TextureView,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(self.entry_point),
            layout: &self.source_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(source),
            }],
        })
    }

    pub fn encode(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        source: &wgpu::BindGroup,
        target: &wgpu::TextureView,
    ) {
        fullscreen_pass::encode(encoder, &self.pipeline, source, target);
    }

    fn new(
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
        fragment: &str,
        entry_point: &'static str,
    ) -> Self {
        let pipeline =
            fullscreen_pass::create_pipeline(device, fragment, entry_point, target_format);
        let source_layout = pipeline.get_bind_group_layout(0);
        Self {
            pipeline,
            source_layout,
            entry_point,
        }
    }
}
