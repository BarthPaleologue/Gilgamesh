extern crate gilgamesh;

use gilgamesh::{init_gilgamesh, start_gilgamesh};
use gilgamesh::mesh::Mesh;

fn main() {
    let mut app = init_gilgamesh();

    let procedural_plane = Mesh::new_procedural_terrain(10.0, 64, &|x: f32, y: f32| x.sin() * y.sin(), &mut app.engine);

    app.scene.add_mesh(procedural_plane);

    start_gilgamesh(app);
}