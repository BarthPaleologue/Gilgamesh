extern crate gilgamesh;

use gilgamesh::engine::Engine;
use gilgamesh::mesh::Mesh;
use gilgamesh::scene::Scene;

fn main() {
    let (mut engine, event_loop) = Engine::new("Gilgamesh", false);

    let mut scene = Scene::new(&engine);

    let sphere = Mesh::new_procedural_sphere("Sphere".into(), 5.0, 32, &|x, y, z| {
        f32::powi(f32::sin(60.0 * x * y * z), 2) * 0.5
    }, 0.5, &mut engine);


    scene.add_mesh(sphere);

    scene.on_before_render.push(Box::new(|engine, meshes| {
        meshes.get_mut("Sphere").unwrap().transform.set_position(0.0, engine.get_elapsed_time().sin(), 0.0);
    }));

    engine.start(scene, event_loop);
}