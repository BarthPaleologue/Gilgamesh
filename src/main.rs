extern crate gilgamesh;

use winit::event::VirtualKeyCode::*;
use gilgamesh::camera::BasicCamera;
use gilgamesh::input::transform_control::OrbitControl;
use gilgamesh::core::engine::Engine;
use gilgamesh::core::scene::Scene;
use gilgamesh::geometry::primitive::PrimitiveMesh;
use gilgamesh::geometry::procedural::ProceduralMesh;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn run() {
    let (mut engine, event_loop) = Engine::new("Gilgamesh", 1000, 800);

    let mut scene = Scene::new(&engine);

    let mut camera = BasicCamera::new(&engine);
    camera.transform.set_position(3.0, 1.5, 3.0);
    camera.control = Some(Box::<OrbitControl>::default());

    scene.set_active_camera(camera);

    let sphere = ProceduralMesh::sphere("Sphere", 4.0, 64, &|x, y, z| {
        f32::powi(f32::sin(60.0 * x * y * z), 2) * 0.5
    }, 0.5, &mut engine);
    let sphere_idx = scene.add_mesh(sphere);


    let sphere2 = ProceduralMesh::sphere("Sphere2", 1.0, 32, &|x, y, z| 0.0, 0.5, &mut engine);
    let sphere2_idx = scene.add_mesh(sphere2);

    let mut plane = PrimitiveMesh::plane("Plane", 10, 10.0, &mut engine);
    plane.transform.set_position(0.0, -5.0, 0.0);
    let plane_idx = scene.add_mesh(plane);

    scene.on_before_render.push(Box::new(move |engine, active_camera, meshes, mouse| {
        meshes[sphere2_idx].transform.set_position(
            7.0 * engine.get_elapsed_time().sin(),
            0.0,
            7.0 * engine.get_elapsed_time().cos(),
        );
    }));

    scene.on_key_pressed.push(Box::new(|engine, active_camera, key| {
        match key {
            T => {
                println!("T pressed at {}", engine.get_elapsed_time());
            }
            _ => {}
        }
    }));

    engine.start(scene, event_loop);
}

fn main() {
    run();
}