# Gilgamesh

[![Rust](https://github.com/BarthPaleologue/Gilgamesh/actions/workflows/rust.yml/badge.svg)](https://github.com/BarthPaleologue/Gilgamesh/actions/workflows/rust.yml)

A small 3D rendering engine built upon WGPU with the primary goal of visualizing procedural terrains.

Loosely based
on [Dr. Xu youtube series](https://www.youtube.com/watch?v=i6WMfY-XTZE&list=PL_UrKDEhALdJS0VrLPn7dqC5A4W1vCAUT)

![Gilgamesh](https://github.com/BarthPaleologue/Gilgamesh/blob/main/cover.png?raw=true)

## Getting Started

Here is a quick example of how to use Gilgamesh to render a procedural terrain:

```rust
extern crate gilgamesh;

use gilgamesh::{init_gilgamesh, start_gilgamesh};
use gilgamesh::mesh::Mesh;

#[test]
fn main() {
    let mut app = init_gilgamesh();

    let procedural_plane = Mesh::new_procedural_terrain(10.0, 64, &|x: f32, y: f32| x.sin() * y.sin(), &mut app.engine);

    app.scene.add_mesh(procedural_plane);

    start_gilgamesh(app);
}
```