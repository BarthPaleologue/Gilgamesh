use crate::core::engine::Engine;
use crate::core::scene::Scene;
use crate::geometry::primitive::PrimitiveMesh;
use crate::lights::light::Light;
use crate::lights::point_light::PointLight;
use crate::materials::blinn_phong::BlinnPhongMaterial;
use crate::transform::Transformable;

pub fn show_point_light_debug_mesh(point_light: &PointLight, scene: &mut Scene, engine: &Engine) -> usize {
    let mut mesh = PrimitiveMesh::sphere("PointLightDebugMesh", 16, engine);
    mesh.transform_mut().parent = Some(point_light.transform_rc());
    mesh.transform_mut().set_scaling(0.5, 0.5, 0.5);
    let (r, g, b) = point_light.color().into();
    let mut material = BlinnPhongMaterial::new("PointLightDebugMaterial", &engine.wgpu_context);
    material.set_ambient_color(r, g, b);
    material.set_diffuse_color(0.0, 0.0, 0.0);
    material.set_specular_color(0.0, 0.0, 0.0);
    mesh.set_material(Box::new(material));
    scene.add_mesh(mesh)
}