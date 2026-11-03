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

The application owns its objects as fields. `Engine<TApp>` creates the application
using a factory with access to `WGPUContext`. Each frame, it calls
`update_before_render` with the time in seconds, mouse input, and GPU context,
then obtains borrowed render data through `get_render_context`.

`RenderContext` borrows the camera and lights and accepts an iterator of meshes;
it does not store or clone the application's objects. Camera controls can be
updated from `FrameContext`, and `on_window_event` handles application-specific
window or keyboard events.

Vectors and matrices use `glam::Vec3` and `glam::Mat4`. Transform rotations use
unit quaternions (`glam::Quat`), and `set_rotation` accepts a quaternion directly.
The camera's `fov` is expressed in degrees.
The camera looks along its local -Z axis, with its quaternion determining its
orientation. `OrbitControl` supplies the orientation towards the orbit's center.

```rust,no_run
use gilgamesh::camera::Camera;
use gilgamesh::core::app::{App, FrameContext};
use gilgamesh::core::engine::Engine;
use gilgamesh::core::render_context::RenderContext;
use gilgamesh::geometry::mesh::Mesh;
use gilgamesh::geometry::procedural::ProceduralMesh;
use gilgamesh::input::transform_control::OrbitControl;
use gilgamesh::lights::directional_light::DirectionalLight;
use gilgamesh::materials::blinn_phong::BlinnPhongMaterial;
use gilgamesh::transform::Transformable;
use glam::Quat;

struct TerrainApp {
    camera: Camera,
    terrain: Mesh,
    directional_light: DirectionalLight,
}

impl App for TerrainApp {
    fn update_before_render(&mut self, frame: &FrameContext<'_>) {
        self.camera.listen_to_control(frame.mouse, frame.delta_seconds);
        self.terrain.transform_mut().set_rotation(Quat::from_rotation_y(frame.elapsed_seconds));
    }

    fn get_render_context(&mut self) -> RenderContext<'_, impl IntoIterator<Item = &mut Mesh>> {
        RenderContext::new(&mut self.camera, [&mut self.terrain], &self.directional_light, &[])
    }
}

fn main() {
    Engine::run("Sinc function", 1000, 800, |ctx| {
        let mut camera = Camera::new(ctx.config.width as f32 / ctx.config.height as f32);
        let mut camera_control = OrbitControl::default();
        camera_control.set_radius(10.0);
        camera.control = Some(Box::new(camera_control));

        let mut terrain = ProceduralMesh::terrain("sinc", 10.0, 128, &|x, z| {
            let d = 5.0 * (x * x + z * z).sqrt();
            3.0 * (f32::sin(d) / d).min(1.0)
        }, ctx);
        let mut material = BlinnPhongMaterial::new("sincMaterial", ctx);
        material.set_diffuse_color(0.5, 0.2, 1.0);
        terrain.set_material(Box::new(material));

        TerrainApp { camera, terrain, directional_light: DirectionalLight::default() }
    });
}
```

## About

Loosely based
on [Dr. Xu youtube series](https://www.youtube.com/watch?v=i6WMfY-XTZE&list=PL_UrKDEhALdJS0VrLPn7dqC5A4W1vCAUT)
and the [WGPU tutorial](https://sotrh.github.io/learn-wgpu/).
