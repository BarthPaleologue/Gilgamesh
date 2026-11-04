use crate::camera::CameraProjection;
use crate::core::frame_bindings::FrameBindings;
use crate::core::render_context::FrameBindingsData;
use crate::core::wgpu_context::WGPUContext;
use crate::geometry::mesh::Mesh;
use crate::materials::material_pipeline::{MaterialPipeline, PipelineConfig, PipelineKey};
use std::collections::HashMap;
use std::iter;
use std::rc::Rc;

pub struct Renderer {
    pub(crate) frame: FrameBindings,
    pipelines: HashMap<PipelineKey, Rc<MaterialPipeline>>,
}

impl Renderer {
    pub fn new(ctx: &WGPUContext) -> Self {
        Self {
            frame: FrameBindings::new(ctx),
            pipelines: HashMap::new(),
        }
    }

    pub(crate) fn material_pipeline(
        &mut self,
        config: PipelineConfig,
        texture_count: usize,
        ctx: &WGPUContext,
    ) -> Rc<MaterialPipeline> {
        let key = PipelineKey {
            config,
            texture_count,
            target_format: ctx.config.format,
        };
        self.pipelines
            .entry(key)
            .or_insert_with(|| {
                Rc::new(MaterialPipeline::new(
                    config,
                    texture_count,
                    &self.frame.layout,
                    ctx,
                ))
            })
            .clone()
    }

    pub(crate) fn update_frame_bindings<'a, ProjectionType>(
        &mut self,
        context: FrameBindingsData<'a, ProjectionType>,
        wgpu_context: &WGPUContext,
    ) where
        ProjectionType: CameraProjection,
    {
        self.frame.update(
            context.camera,
            context.directional_light,
            context.point_lights,
            &wgpu_context.queue,
        );
    }

    pub(crate) fn render<'a, RenderObjects>(
        &mut self,
        render_objects: RenderObjects,
        wgpu_context: &WGPUContext,
        output: &wgpu::SurfaceTexture,
    ) where
        RenderObjects: IntoIterator<Item = &'a mut Mesh>,
    {
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

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
                    view: &wgpu_context.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0.0),
                        store: wgpu::StoreOp::Discard,
                    }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });

            self.frame.bind(&mut render_pass);
            for mesh in render_objects {
                mesh.render(&mut render_pass, self, wgpu_context);
            }
        }

        wgpu_context.queue.submit(iter::once(encoder.finish()));
    }
}
