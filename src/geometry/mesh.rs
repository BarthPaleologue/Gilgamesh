use crate::core::object_bindings::ObjectBindings;
use crate::core::renderer::Renderer;
use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

use crate::core::wgpu_context::WGPUContext;
use crate::geometry::cpu_mesh_buffers::CpuMeshBuffers;
use crate::geometry::gpu_mesh_buffers::GpuMeshBuffers;
use wgpu::RenderPass;

use crate::materials::material::Material;
use crate::transform::{Transform, Transformable};

pub struct Mesh {
    transform: Rc<RefCell<Transform>>,
    gpu_buffers: GpuMeshBuffers,
    object_bindings: ObjectBindings,
    material: Rc<dyn Material>,
}

impl Transformable for Mesh {
    fn transform(&self) -> Ref<'_, Transform> {
        self.transform.borrow()
    }
    fn transform_mut(&self) -> RefMut<'_, Transform> {
        self.transform.borrow_mut()
    }
    fn transform_rc(&self) -> Rc<RefCell<Transform>> {
        self.transform.clone()
    }
}

impl Mesh {
    /// Uploads the geometry and releases the CPU buffers after construction.
    pub fn from_cpu_buffers(
        name: &str,
        cpu_buffers: CpuMeshBuffers,
        material: Rc<dyn Material>,
        wgpu_context: &WGPUContext,
    ) -> Mesh {
        let gpu_buffers = GpuMeshBuffers::from_cpu_buffers(&cpu_buffers, wgpu_context);
        Self::from_gpu_buffers(name, gpu_buffers, material, wgpu_context)
    }

    /// Uses existing GPU geometry without uploading or retaining CPU data.
    pub fn from_gpu_buffers(
        name: &str,
        gpu_buffers: GpuMeshBuffers,
        material: Rc<dyn Material>,
        wgpu_context: &WGPUContext,
    ) -> Mesh {
        Mesh {
            transform: Rc::new(RefCell::new(Transform::new())),
            gpu_buffers,
            object_bindings: ObjectBindings::new(name, wgpu_context),
            material,
        }
    }

    pub fn set_material(&mut self, material: Rc<dyn Material>) {
        self.material = material;
    }

    pub fn render(
        &mut self,
        render_pass: &mut RenderPass<'_>,
        renderer: &mut Renderer,
        ctx: &WGPUContext,
    ) {
        self.object_bindings
            .update(&self.transform.borrow(), &ctx.queue);
        self.material.prepare(renderer, ctx).bind(render_pass);
        self.object_bindings.bind(render_pass);
        self.gpu_buffers.draw(render_pass);
    }
}
