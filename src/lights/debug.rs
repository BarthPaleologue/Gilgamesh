use crate::core::wgpu_context::WGPUContext;
use crate::geometry::mesh::Mesh;
use crate::geometry::primitive;
use crate::lights::light::Light;
use crate::lights::point_light::PointLight;
use crate::materials::blinn_phong::BlinnPhongMaterial;
use crate::transform::Transformable;
use glam::DVec3;

pub fn create_point_light_debug_mesh(point_light: &PointLight, wgpu_context: &WGPUContext) -> Mesh {
    let mut mesh =
        Mesh::from_cpu_buffers("PointLightDebugMesh", primitive::sphere(16), wgpu_context);
    mesh.transform_mut().parent = Some(point_light.transform_rc());
    mesh.transform_mut().scaling = DVec3::splat(0.5);
    let (r, g, b) = point_light.color().into();
    let mut material = BlinnPhongMaterial::new("PointLightDebugMaterial", wgpu_context);
    material.set_ambient_color(r, g, b);
    material.set_diffuse_color(0.0, 0.0, 0.0);
    material.set_specular_color(0.0, 0.0, 0.0);
    mesh.set_material(Box::new(material));
    mesh
}
