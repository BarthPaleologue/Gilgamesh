extern crate gilgamesh;

use std::rc::Rc;
use gilgamesh::{init_gilgamesh, start_gilgamesh};
use gilgamesh::material::Material;
use gilgamesh::mesh::Mesh;

fn main() {
    let mut app = init_gilgamesh();

    let sphere = Mesh::new_procedural_sphere(10.0, 20, &|x, y, z| {
        f32::powi(f32::sin(100.0 * x * y * z), 2) / 2.0
    }, &mut app.engine);

    app.scene.add_mesh(sphere);

    start_gilgamesh(app);
}