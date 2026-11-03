use crate::camera::camera::Camera;
use crate::core::engine::Engine;
use crate::core::scene::Scene;
use crate::geometry::procedural::ProceduralMesh;
use crate::input::transform_control::OrbitControl;
use crate::materials::blinn_phong::BlinnPhongMaterial;

pub fn run() {
    Engine::run("Sinc function", 1000, 800, create_scene);
}

fn create_scene(engine: &Engine) -> Scene {
    let mut scene = Scene::new(engine);

    let mut camera = Camera::new(engine);
    let mut camera_control = OrbitControl::default();
    camera_control.set_radius(10.0);
    camera.control = Some(Box::new(camera_control));
    scene.set_active_camera(camera);

    let mut mesh = ProceduralMesh::terrain(
        "sinc",
        10.0,
        128,
        &|x, z| {
            let d = 5.0 * (x * x + z * z).sqrt();
            3.0 * (f32::sin(d) / d).min(1.0)
        },
        engine,
    );
    let mut material = BlinnPhongMaterial::new("sincMaterial", &engine.wgpu_context);
    material.set_diffuse_color(0.5, 0.2, 1.0);
    mesh.set_material(Box::new(material));
    scene.add_mesh(mesh);

    scene
}
