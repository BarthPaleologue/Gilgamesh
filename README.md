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

Transforms and camera calculations use `f64`, as do frame times. Vectors and
matrices use `glam::DVec3` and `glam::DMat4`, and rotations use unit quaternions
(`glam::DQuat`). Transform fields are public: assign positions with
`transform.position = DVec3::new(x, y, z)` and uniform scales with
`transform.scaling = DVec3::splat(scale)`. Colors remain `f32`.
Vertex attributes use `f32` on both CPU and GPU, with `u32` triangle indices.
Transform and camera uniforms are converted to `f32` when preparing GPU buffers.
Shaders use `f32`.

The `geometry::primitive` and `geometry::procedural` functions return
`CpuMeshBuffers`, containing per-vertex attribute arrays in `CpuVertexBuffers`
and a separate array of triangle indices. Both are editable before upload.
They do not need a GPU context. Pass these buffers to
`Mesh::from_cpu_buffers(name, buffers, material, ctx)` to create a renderable mesh.
`Mesh` consumes the CPU data and retains only `GpuMeshBuffers`, which hold the
GPU buffers and index count. `GpuMeshBuffers::from_cpu_buffers(&buffers, ctx)`
can also upload geometry while leaving the CPU data with its caller.

Materials are configured before sharing them as `Rc<dyn Material>`. Rendering
only borrows them through `&self`: no `RefCell` is required. A private `OnceCell`
initializes each material's GPU bindings on first use. Setters invalidate these
bindings when the material is exclusively owned; shared materials are replaced
rather than modified in place. Each mesh owns its transform buffer, while the
renderer uploads camera and lighting data once per frame.

The same material can be used by meshes with independent transforms:

```rust,no_run
use gilgamesh::core::wgpu_context::WGPUContext;
use gilgamesh::geometry::{mesh::Mesh, primitive};
use gilgamesh::materials::blinn_phong::BlinnPhongMaterial;
use gilgamesh::transform::Transformable;
use glam::DVec3;
use std::rc::Rc;

fn spheres(ctx: &WGPUContext) -> (Mesh, Mesh) {
    let mut material = BlinnPhongMaterial::new("shared", ctx);
    material.set_diffuse_color(0.2, 0.5, 1.0);
    let material = Rc::new(material);
    let left = Mesh::from_cpu_buffers("left", primitive::sphere(32), material.clone(), ctx);
    let right = Mesh::from_cpu_buffers("right", primitive::sphere(32), material, ctx);
    left.transform_mut().position = DVec3::new(-2.0, 0.0, 0.0);
    right.transform_mut().position = DVec3::new(2.0, 0.0, 0.0);
    (left, right)
}
```

The camera's `fov` is expressed in degrees.
The camera looks along its local -Z axis, with its quaternion determining its
orientation. `OrbitControl` supplies the orientation towards the orbit's center.

```rust,no_run
use gilgamesh::camera::Camera;
use gilgamesh::core::app::{App, FrameContext};
use gilgamesh::core::engine::Engine;
use gilgamesh::core::render_context::RenderContext;
use gilgamesh::geometry::mesh::Mesh;
use gilgamesh::geometry::procedural;
use gilgamesh::input::transform_control::OrbitControl;
use gilgamesh::lights::directional_light::DirectionalLight;
use gilgamesh::materials::blinn_phong::BlinnPhongMaterial;
use gilgamesh::transform::Transformable;
use glam::DQuat;
use std::rc::Rc;

struct TerrainApp {
    camera: Camera,
    terrain: Mesh,
    directional_light: DirectionalLight,
}

impl App for TerrainApp {
    fn update_before_render(&mut self, frame: &FrameContext<'_>) {
        self.camera.listen_to_control(frame.mouse, frame.delta_seconds);
        self.terrain.transform_mut().rotation = DQuat::from_rotation_y(frame.elapsed_seconds);
    }

    fn get_render_context(&mut self) -> RenderContext<'_, impl IntoIterator<Item = &mut Mesh>> {
        RenderContext::new(&mut self.camera, [&mut self.terrain], &self.directional_light, &[])
    }
}

fn main() {
    Engine::run("Sinc function", 1000, 800, |ctx| {
        let mut camera = Camera::new(ctx.config.width as f64 / ctx.config.height as f64);
        let mut camera_control = OrbitControl::default();
        camera_control.set_radius(10.0);
        camera.control = Some(Box::new(camera_control));

        let cpu_buffers = procedural::terrain(10.0, 128, &|x, z| {
            let d = 5.0 * (x * x + z * z).sqrt();
            3.0 * (d.sin() / d).min(1.0)
        });
        let mut material = BlinnPhongMaterial::new("sincMaterial", ctx);
        material.set_diffuse_color(0.5, 0.2, 1.0);
        let terrain = Mesh::from_cpu_buffers("sinc", cpu_buffers, Rc::new(material), ctx);

        TerrainApp { camera, terrain, directional_light: DirectionalLight::default() }
    });
}
```

## About

Loosely based
on [Dr. Xu youtube series](https://www.youtube.com/watch?v=i6WMfY-XTZE&list=PL_UrKDEhALdJS0VrLPn7dqC5A4W1vCAUT)
and the [WGPU tutorial](https://sotrh.github.io/learn-wgpu/).
