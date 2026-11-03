use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

use crate::camera::Camera;
use crate::core::wgpu_context::WGPUContext;
use crate::geometry::cpu_mesh_buffers::CpuMeshBuffers;
use crate::geometry::gpu_mesh_buffers::GpuMeshBuffers;
use crate::lights::directional_light::DirectionalLight;
use crate::lights::point_light::PointLight;
use wgpu::RenderPass;

use crate::materials::blinn_phong::BlinnPhongMaterial;
use crate::materials::material::Material;
use crate::transform::{Transform, Transformable};

pub struct Mesh {
    transform: Rc<RefCell<Transform>>,
    gpu_buffers: GpuMeshBuffers,
    material: Box<dyn Material>,
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
        wgpu_context: &WGPUContext,
    ) -> Mesh {
        let gpu_buffers = GpuMeshBuffers::from_cpu_buffers(&cpu_buffers, wgpu_context);

        Mesh {
            transform: Rc::new(RefCell::new(Transform::new())),
            gpu_buffers,
            material: Box::new(BlinnPhongMaterial::new(
                &format!("{}Material", name),
                wgpu_context,
            )),
        }
    }

    pub fn set_material(&mut self, material: Box<dyn Material>) {
        self.material = material;
    }

    /// you may be asking wtf is going on with the lifetimes here, and I don't know either. Dark magic.
    pub fn render<'a>(
        &'a mut self,
        render_pass: &mut RenderPass<'a>,
        active_camera: &Camera,
        directional_light: &DirectionalLight,
        point_lights: &[PointLight],
        wgpu_context: &WGPUContext,
    ) {
        let transform = self.transform_rc();
        self.material.bind(
            render_pass,
            transform.borrow(),
            active_camera,
            point_lights,
            directional_light,
            wgpu_context,
        );

        self.gpu_buffers.draw(render_pass);
    }
}
