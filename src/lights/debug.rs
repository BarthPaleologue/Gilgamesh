use crate::core::wgpu_context::WGPUContext;
use crate::geometry::mesh::Mesh;
use crate::geometry::primitive;
use crate::lights::light::Light;
use crate::lights::point_light::PointLight;
use crate::materials::blinn_phong::BlinnPhongMaterial;
use crate::transform::Transformable;
use std::rc::Rc;

pub fn create_point_light_debug_mesh(point_light: &PointLight, wgpu_context: &WGPUContext) -> Mesh {
    let (r, g, b) = point_light.color().into();
    let intensity = point_light.intensity().value();
    let mut material = BlinnPhongMaterial::new("PointLightDebugMaterial");
    material.set_ambient_color(r * intensity, g * intensity, b * intensity);
    material.set_diffuse_color(0.0, 0.0, 0.0);
    material.set_specular_color(0.0, 0.0, 0.0);
    let mut buffers = primitive::sphere(16);
    for position in &mut buffers.vertices.positions {
        for coordinate in position {
            *coordinate *= 0.5;
        }
    }
    let mesh = Mesh::from_cpu_buffers(
        "PointLightDebugMesh",
        &buffers,
        Rc::new(material.into()),
        wgpu_context,
    );
    mesh.transform_mut().parent = Some(point_light.transform_rc());
    mesh
}
