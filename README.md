# Gilgamesh

![Gilgamesh Logo](logo.png)

[![Rust](https://github.com/BarthPaleologue/Gilgamesh/actions/workflows/rust.yml/badge.svg)](https://github.com/BarthPaleologue/Gilgamesh/actions/workflows/rust.yml)
[![Crates.io](https://img.shields.io/crates/v/gilgamesh)](https://crates.io/crates/gilgamesh)
[![Crates.io](https://img.shields.io/crates/l/gilgamesh)]()

A small 3D WGPU engine written in Rust that does not rely on the ECS pattern.

## Features

- Up to 128 point lights per material
- Up to 128 directional lights per material
- Built-in Phong shading
- Normal mapping
- Procedural mesh generation

## Coming Soon

- Shadow mapping
- PBR shading
- GLTF support

![sinc function](cover.png)

## Getting Started

Requires Rust 1.99 or newer. `rust-toolchain.toml` selects the stable toolchain,
Rustfmt, Clippy, and the `wasm32-unknown-unknown` target.

```sh
cargo build --locked
cargo test --locked
cargo build --locked --target wasm32-unknown-unknown
```

Here is a quick example of how to use Gilgamesh to render a procedural terrain:

```rust
use gilgamesh::camera::camera::Camera;
use gilgamesh::core::engine::Engine;
use gilgamesh::core::scene::Scene;
use gilgamesh::geometry::procedural::ProceduralMesh;
use gilgamesh::input::transform_control::OrbitControl;
use gilgamesh::materials::blinn_phong::BlinnPhongMaterial;

fn main() {
    Engine::run("Sinc function", 1000, 800, |engine| {
        let mut scene = Scene::new(engine);

        let mut camera = Camera::new(engine);
        let mut camera_control = OrbitControl::default();
        camera_control.set_radius(2.0);
        camera.control = Some(Box::new(camera_control));
        scene.set_active_camera(camera);

        let mut mesh = ProceduralMesh::terrain("sinc", 10.0, 128, &|x, z| {
            let d = 5.0 * (x * x + z * z).sqrt();
            3.0 * (f32::sin(d) / d).min(1.0)
        }, engine);
        let mut material = BlinnPhongMaterial::new("sincMaterial", &engine.wgpu_context);
        material.set_diffuse_color(0.5, 0.2, 1.0);
        mesh.set_material(Box::new(material));
        scene.add_mesh(mesh);

        scene
    });
}
```

## About

Loosely based
on [Dr. Xu youtube series](https://www.youtube.com/watch?v=i6WMfY-XTZE&list=PL_UrKDEhALdJS0VrLPn7dqC5A4W1vCAUT)
and the [WGPU tutorial](https://sotrh.github.io/learn-wgpu/).
