use crate::core::frame_bindings::FrameBindings;
use crate::core::material_pipeline::{MaterialPipeline, PipelineConfig};
use crate::core::object_bindings::ObjectBindings;
use crate::core::prepared_material::PreparedMaterial;
use crate::core::render_context::FrameBindingsData;
use crate::core::render_target::RenderTarget;
use crate::core::wgpu_context::WGPUContext;
use crate::geometry::mesh::Mesh;
use crate::materials::pbr::PbrMaterial;
use crate::texture::{Linear, Texture};
use crate::transform::{Transform, Transformable};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};

struct CachedMaterial {
    source: Weak<PbrMaterial>,
    prepared: PreparedMaterial,
}

struct CachedTransform {
    source: Weak<RefCell<Transform>>,
    bindings: ObjectBindings,
}

#[must_use]
pub struct RasterRenderer {
    pub(crate) frame: FrameBindings,
    pipelines: HashMap<PipelineConfig, Rc<MaterialPipeline>>,
    materials: HashMap<*const PbrMaterial, CachedMaterial>,
    transforms: HashMap<*const RefCell<Transform>, CachedTransform>,
    object_layout: wgpu::BindGroupLayout,
    pub(crate) fallback_texture: Texture<Linear>,
    polygon_mode: wgpu::PolygonMode,
    target_format: wgpu::TextureFormat,
}

impl RasterRenderer {
    pub fn new(ctx: &WGPUContext, target_format: wgpu::TextureFormat) -> Self {
        Self {
            frame: FrameBindings::new(ctx),
            pipelines: HashMap::new(),
            materials: HashMap::new(),
            transforms: HashMap::new(),
            object_layout: ObjectBindings::create_layout(&ctx.device),
            fallback_texture: Texture::new_empty("Fallback Texture", &ctx.device),
            polygon_mode: wgpu::PolygonMode::Fill,
            target_format,
        }
    }

    pub fn set_polygon_mode(&mut self, polygon_mode: wgpu::PolygonMode) {
        if self.polygon_mode != polygon_mode {
            self.polygon_mode = polygon_mode;
            self.materials.clear();
        }
    }

    pub(crate) fn material_pipeline(
        &mut self,
        config: PipelineConfig,
        ctx: &WGPUContext,
    ) -> Rc<MaterialPipeline> {
        self.pipelines
            .entry(config)
            .or_insert_with(|| {
                Rc::new(MaterialPipeline::new(
                    config,
                    self.target_format,
                    &self.frame.layout,
                    &self.object_layout,
                    ctx,
                ))
            })
            .clone()
    }

    pub fn update_frame_bindings(
        &mut self,
        context: &FrameBindingsData<'_>,
        aspect_ratio: f64,
        wgpu_context: &WGPUContext,
    ) {
        self.frame.update(
            &context.camera_uniforms(aspect_ratio),
            context.directional_lights,
            context.point_lights,
            &wgpu_context.queue,
        );
    }

    /// Records a scene pass into the caller's encoder using the supplied attachments.
    pub fn encode<'a, RenderObjects>(
        &mut self,
        render_objects: RenderObjects,
        wgpu_context: &WGPUContext,
        encoder: &mut wgpu::CommandEncoder,
        target: &RenderTarget<'_>,
        clear_color: [f32; 4],
    ) where
        RenderObjects: IntoIterator<Item = &'a Mesh>,
    {
        self.materials
            .retain(|_, cached| cached.source.strong_count() > 0);
        self.transforms
            .retain(|_, cached| cached.source.strong_count() > 0);
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target.color,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: f64::from(clear_color[0]),
                            g: f64::from(clear_color[1]),
                            b: f64::from(clear_color[2]),
                            a: f64::from(clear_color[3]),
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: target.depth,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });

            self.frame.bind(&mut render_pass);
            for mesh in render_objects {
                self.render_mesh(mesh, &mut render_pass, wgpu_context);
            }
        }
    }

    fn render_mesh(&mut self, mesh: &Mesh, pass: &mut wgpu::RenderPass<'_>, ctx: &WGPUContext) {
        let material = mesh.material();
        let material_key = Rc::as_ptr(material);
        // The weak reference keeps the allocation's identity stable and makes edits
        // through Rc::make_mut detach from any previously prepared description.
        if !self.materials.contains_key(&material_key) {
            let prepared = PreparedMaterial::new(material, self.polygon_mode, self, ctx);
            self.materials.insert(
                material_key,
                CachedMaterial {
                    source: Rc::downgrade(material),
                    prepared,
                },
            );
        }

        let transform = mesh.transform_rc();
        let object = self
            .transforms
            .entry(Rc::as_ptr(&transform))
            .or_insert_with(|| CachedTransform {
                source: Rc::downgrade(&transform),
                bindings: ObjectBindings::new(mesh.name(), &self.object_layout, ctx),
            });
        object.bindings.update(&transform.borrow(), &ctx.queue);
        self.materials[&material_key].prepared.bind(pass);
        object.bindings.bind(pass);
        let geometry = mesh.geometry();
        pass.set_vertex_buffer(0, geometry.vertex_buffer().slice(..));
        pass.set_index_buffer(geometry.index_buffer().slice(..), wgpu::IndexFormat::Uint32);
        pass.draw_indexed(0..geometry.index_count(), 0, 0..1);
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests;
