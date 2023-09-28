extern crate gilgamesh;

use gilgamesh::engine::Engine;
use gilgamesh::mesh::Mesh;
use gilgamesh::scene::Scene;

fn main() {
    let (mut engine, event_loop) = Engine::new("Gilgamesh", false);

    let mut scene = Scene::new(&engine);

    let mut sphere = Mesh::new_procedural_sphere(5.0, 32, &|x, y, z| {
        f32::powi(f32::sin(60.0 * x * y * z), 2) * 0.5
    }, 0.5, &mut engine);


    scene.add_mesh(sphere);

    engine.on_before_render.push(Box::new(|_| {
        println!("Before render");
    }));

    engine.on_before_render.push(Box::new(|_| {
        println!("Also Before render");
    }));

    engine.start(scene, event_loop);
}