use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

use crate::core::wgpu_context::WGPUContext;
use crate::geometry::cpu_mesh_buffers::CpuMeshBuffers;
use crate::geometry::gpu_mesh_buffers::GpuMeshBuffers;

use crate::materials::material::Material;
use crate::transform::{Transform, Transformable};

pub struct Mesh {
    name: String,
    transform: Rc<RefCell<Transform>>,
    gpu_buffers: GpuMeshBuffers,
    material: Rc<Material>,
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
        material: Rc<Material>,
        wgpu_context: &WGPUContext,
    ) -> Mesh {
        let gpu_buffers = GpuMeshBuffers::from_cpu_buffers(&cpu_buffers, wgpu_context);
        Self::from_gpu_buffers(name, gpu_buffers, material)
    }

    /// Uses existing GPU geometry without uploading or retaining CPU data.
    pub fn from_gpu_buffers(
        name: &str,
        gpu_buffers: GpuMeshBuffers,
        material: Rc<Material>,
    ) -> Mesh {
        Mesh {
            name: name.to_string(),
            transform: Rc::new(RefCell::new(Transform::new())),
            gpu_buffers,
            material,
        }
    }

    pub fn set_material(&mut self, material: Rc<Material>) {
        self.material = material;
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn material(&self) -> &Rc<Material> {
        &self.material
    }

    pub fn geometry(&self) -> &GpuMeshBuffers {
        &self.gpu_buffers
    }
}
