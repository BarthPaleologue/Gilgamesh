use crate::camera::camera::Camera;
use crate::input::transform_control::OrbitControl;
use crate::core::engine::Engine;
use crate::core::scene::Scene;
use crate::geometry::procedural::ProceduralMesh;
use crate::materials::blinn_phong::BlinnPhongMaterial;
use crate::transform::{Transformable};

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn run() {
    let (mut engine, event_loop) = Engine::new("Sinc function", 1000, 800);

    let mut scene = Scene::new(&engine);

    let mut camera = Camera::new(&engine);
    let mut camera_control = OrbitControl::default();
    camera_control.set_radius(10.0);
    camera.control = Some(Box::new(camera_control));
    scene.set_active_camera(camera);

    let mut mesh = ProceduralMesh::terrain("sinc", 10.0, 128, &|x, z| {
        let d = 5.0 * (x * x + z * z).sqrt();
        3.0 * (f32::sin(d) / d).min(1.0)
    }, &engine);
    let mut material = BlinnPhongMaterial::new("sincMaterial", &engine.wgpu_context);
    material.set_diffuse_color(0.5, 0.2, 1.0);
    mesh.set_material(Box::new(material));
    scene.add_mesh(mesh);

    engine.start(scene, event_loop);
}