use crate::core::frame_bindings::FrameBindings;
use crate::core::render_context::RenderContext;
use crate::core::wgpu_context::WGPUContext;
use crate::geometry::mesh::Mesh;
use std::iter;

pub struct Renderer {
    pub(crate) frame: FrameBindings,
}

impl Renderer {
    pub fn new(ctx: &WGPUContext) -> Self {
        Self {
            frame: FrameBindings::new(ctx),
        }
    }

    pub(crate) fn render<'a, T>(
        &mut self,
        context: RenderContext<'a, T>,
        wgpu_context: &WGPUContext,
        output: &wgpu::SurfaceTexture,
    ) where
        T: IntoIterator<Item = &'a mut Mesh>,
    {
        self.frame.update(
            context.camera,
            context.directional_light,
            context.point_lights,
            &wgpu_context.queue,
        );
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let depth_texture = wgpu_context
            .device
            .create_texture(&wgpu::TextureDescriptor {
                size: wgpu::Extent3d {
                    width: wgpu_context.config.width,
                    height: wgpu_context.config.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth24Plus,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                label: None,
                view_formats: &[],
            });
        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder =
            wgpu_context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.1,
                            b: 0.2,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Discard,
                    }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });

            self.frame.bind(&mut render_pass);
            for mesh in context.render_objects {
                mesh.render(&mut render_pass, self, wgpu_context);
            }
        }

        wgpu_context.queue.submit(iter::once(encoder.finish()));
    }
}
