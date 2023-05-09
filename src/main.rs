extern crate gilgamesh;

use std::rc::Rc;
use gilgamesh::{init_gilgamesh, start_gilgamesh};
use gilgamesh::material::Material;
use gilgamesh::mesh::Mesh;

fn main() {
    let mut app = init_gilgamesh();

    let mut procedural_plane = Mesh::new_procedural_terrain(10.0, 64, &|x: f32, y: f32| x.sin() * y.sin() * 0.5 + 0.5, 1.0, &mut app.engine);

    app.scene.add_mesh(procedural_plane);

    start_gilgamesh(app);
}