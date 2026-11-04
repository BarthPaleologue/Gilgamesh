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

Run the examples on desktop:

```sh
cargo demo terrain
cargo demo starsystem
```

For the browser, install [wasm-server-runner](https://github.com/jakobhellermann/wasm-server-runner)
once with `cargo install wasm-server-runner`, then run either example and open the printed URL:

```sh
cargo demo-wasm terrain
cargo demo-wasm starsystem
```

Implement `App` to own and update your camera, controls, and meshes.
Here is a procedural terrain example:

```rust,no_run
use gilgamesh::camera::{Camera, CameraProjection, Perspective};
use gilgamesh::core::app::{App, FrameContext};
use gilgamesh::core::engine::Engine;
use gilgamesh::core::render_context::FrameBindingsData;
use gilgamesh::core::wgpu_context::WGPUContext;
use gilgamesh::geometry::mesh::Mesh;
use gilgamesh::geometry::procedural;
use gilgamesh::input::orbit_control::OrbitControl;
use gilgamesh::lights::directional_light::DirectionalLight;
use gilgamesh::materials::blinn_phong::BlinnPhongMaterial;
use gilgamesh::transform::Transformable;
use glam::DQuat;
use std::rc::Rc;

struct TerrainApp {
    camera: Camera<Perspective>,
    orbit_control: OrbitControl,
    terrain: Mesh,
    directional_light: DirectionalLight,
}

impl App for TerrainApp {
    fn new(ctx: &WGPUContext) -> Self {
        let camera = Camera::default();
        let mut orbit_control = OrbitControl::default();
        orbit_control.set_radius(10.0);

        let cpu_buffers = procedural::terrain(10.0, 128, &|x, z| {
            let d = 5.0 * (x * x + z * z).sqrt();
            3.0 * (d.sin() / d).min(1.0)
        });
        let mut material = BlinnPhongMaterial::new("sincMaterial");
        material.set_diffuse_color(0.5, 0.2, 1.0);
        let terrain = Mesh::from_cpu_buffers("sinc", cpu_buffers, Rc::new(material.into()), ctx);

        Self { camera, orbit_control, terrain, directional_light: DirectionalLight::default() }
    }

    fn update_before_render(&mut self, frame: &FrameContext<'_>) {
        self.camera.projection.aspect_ratio = frame.render_width as f64 / frame.render_height as f64;
        self.orbit_control.update(&mut self.camera, frame.mouse, frame.delta_seconds);
        self.terrain.transform_mut().rotation = DQuat::from_rotation_y(frame.elapsed_seconds);
    }

    fn get_meshes(&self) -> impl IntoIterator<Item = &Mesh> {
        [&self.terrain]
    }

    fn get_frame_bindings_data(&mut self) -> FrameBindingsData<'_, impl CameraProjection> {
        FrameBindingsData::new(&self.camera, &self.directional_light, &[])
    }
}

fn main() {
    Engine::<TerrainApp>::run("Sinc function", 1000, 800);
}
```

## About

Loosely based
on [Dr. Xu youtube series](https://www.youtube.com/watch?v=i6WMfY-XTZE&list=PL_UrKDEhALdJS0VrLPn7dqC5A4W1vCAUT)
and the [WGPU tutorial](https://sotrh.github.io/learn-wgpu/).
