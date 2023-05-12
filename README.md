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

fn main() {
    let mut app = init_gilgamesh();

    let sphere = Mesh::new_procedural_sphere(5.0, 32, &|x, y, z| {
        f32::powi(f32::sin(60.0 * x * y * z), 2) / 2.0
    }, 0.5, &mut app.engine);

    app.scene.add_mesh(sphere);

    start_gilgamesh(app);
}
```