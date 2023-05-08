extern crate core;
extern crate gilgamesh;

use cgmath::{InnerSpace, Rotation3};
use winit::{
    event::*,
    event_loop::{EventLoop, ControlFlow},
    window::{WindowBuilder},
};
use cgmath::num_traits::ToPrimitive;
use gilgamesh::{init_gilgamesh, start_gilgamesh};
use gilgamesh::mesh::Mesh;
use gilgamesh::camera::*;

#[test]
fn main() {
    let mut app = init_gilgamesh();

    let mut procedural_plane = Mesh::new_procedural_terrain(10.0, 64, &|x: f32, y: f32| x.sin() * y.sin(), &mut app.engine);

    let mut plane = app.scene.add_mesh(procedural_plane);

    start_gilgamesh(app);
}