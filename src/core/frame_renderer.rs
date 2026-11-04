use crate::camera::CameraProjection;
use crate::core::color_pass::ColorPass;
use crate::core::raster_renderer::RasterRenderer;
use crate::core::render_context::FrameBindingsData;
use crate::core::render_target::{OffscreenTarget, RenderTarget, create_color_view};
use crate::core::wgpu_context::WGPUContext;
use crate::geometry::mesh::Mesh;

const SCENE_COLOR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
const LDR_IMAGE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

/// Renders borrowed scene data into offscreen images, independently of an application or window.
pub struct FrameRenderer {
    wgpu_context: WGPUContext,
    renderer: RasterRenderer,
    tone_mapping: ColorPass,
}

impl FrameRenderer {
    /// Initializes a GPU context without a presentation surface.
    pub async fn new() -> Self {
        let instance = wgpu::util::new_instance_with_webgpu_detection(
            wgpu::InstanceDescriptor::new_without_display_handle(),
        )
        .await;
        Self::from_context(WGPUContext::new(instance, None).await)
    }

    /// Uses an existing context, including one selected for compatibility with a surface.
    pub fn from_context(ctx: WGPUContext) -> Self {
        assert!(
            ctx.adapter
                .get_texture_format_features(SCENE_COLOR_FORMAT)
                .allowed_usages
                .contains(
                    wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING
                ),
            "HDR rendering requires Rgba16Float color attachments; WebGL needs floating-point color buffer support",
        );
        let tone_mapping = ColorPass::tone_mapping(&ctx.device, LDR_IMAGE_FORMAT);
        Self {
            renderer: RasterRenderer::new(&ctx, SCENE_COLOR_FORMAT),
            tone_mapping,
            wgpu_context: ctx,
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
        FrameTarget::new(
            &self.wgpu_context.device,
            &self.tone_mapping,
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        )
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

    /// Submits the scene and tone mapping passes. The caller prepares the scene and camera.
    pub fn render_to<'a, ProjectionType, RenderObjects>(
        &mut self,
        target: &FrameTarget,
        bindings: FrameBindingsData<'_, ProjectionType>,
        meshes: RenderObjects,
    ) where
        ProjectionType: CameraProjection,
        RenderObjects: IntoIterator<Item = &'a Mesh>,
    {
        let mut encoder =
            self.wgpu_context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Frame Render Encoder"),
                });
        self.encode(target, bindings, meshes, &mut encoder);
        self.wgpu_context.queue.submit([encoder.finish()]);
    }

    pub(crate) fn encode<'a, ProjectionType, RenderObjects>(
        &mut self,
        target: &FrameTarget,
        bindings: FrameBindingsData<'_, ProjectionType>,
        meshes: RenderObjects,
        encoder: &mut wgpu::CommandEncoder,
    ) where
        ProjectionType: CameraProjection,
        RenderObjects: IntoIterator<Item = &'a Mesh>,
    {
        let ctx = &self.wgpu_context;
        self.renderer.update_frame_bindings(bindings, ctx);
        self.renderer.encode(
            meshes,
            ctx,
            encoder,
            RenderTarget {
                color: &target.scene.color,
                depth: &target.scene.depth,
            },
        );
        self.tone_mapping
            .encode(encoder, &target.hdr_source, &target.ldr_image);
    }
}

/// Owns the HDR scene attachments and the final `Rgba8UnormSrgb` image for one render size.
pub struct FrameTarget {
    scene: OffscreenTarget,
    ldr_image: wgpu::TextureView,
    hdr_source: wgpu::BindGroup,
}

impl FrameTarget {
    fn new(device: &wgpu::Device, tone_mapping: &ColorPass, size: wgpu::Extent3d) -> Self {
        let scene = OffscreenTarget::new(device, size, SCENE_COLOR_FORMAT);
        let ldr_image = create_color_view(device, "LDR Image", size, LDR_IMAGE_FORMAT);
        let hdr_source = tone_mapping.bind_source(device, &scene.color);
        Self {
            scene,
            ldr_image,
            hdr_source,
        }
    }

    /// The final image, available for presentation, readback, or another GPU pass.
    pub fn output(&self) -> &wgpu::TextureView {
        &self.ldr_image
    }

    /// Starts copying the last rendered image without drawing or advancing the application.
    /// The returned future owns the readback resources, so this target can be reused immediately.
    /// Browser readback requires WebGPU; WebGL returns `Unsupported`.
    pub fn read_image(
        &self,
        ctx: &WGPUContext,
    ) -> impl std::future::Future<Output = std::io::Result<image::RgbaImage>> + use<> {
        crate::core::readback::read_rgba8(&ctx.device, &ctx.queue, self.output().texture())
    }
}
