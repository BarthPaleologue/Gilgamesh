extern crate gilgamesh;

use gilgamesh::app::App;
use gilgamesh::mesh::Mesh;
use gilgamesh::scene::Scene;

fn main() {
    let (mut app, event_loop) = App::new();

    let mut scene = Scene::new(&app);

    let sphere = Mesh::new_procedural_sphere(5.0, 32, &|x, y, z| {
        f32::powi(f32::sin(60.0 * x * y * z), 2) * 0.5
    }, 0.5, &mut app);

    scene.add_mesh(sphere);

    app.start(scene, event_loop);
}