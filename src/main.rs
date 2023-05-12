extern crate gilgamesh;

use gilgamesh::{init_gilgamesh, start_gilgamesh};
use gilgamesh::mesh::Mesh;

fn main() {
    let mut app = init_gilgamesh();

    let sphere = Mesh::new_procedural_sphere(5.0, 32, &|x, y, z| {
        f32::powi(f32::sin(60.0 * x * y * z), 2) * 0.5
    }, 0.5, &mut app.engine);

    app.scene.add_mesh(sphere);

    start_gilgamesh(app);
}