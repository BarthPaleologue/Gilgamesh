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

Here is a quick example of how to use Gilgamesh to render a procedural terrain:

```rust
extern crate gilgamesh;
use crate::camera::camera::Camera;
use crate::input::transform_control::OrbitControl;
use crate::core::engine::Engine;
use crate::core::scene::Scene;
use crate::geometry::procedural::ProceduralMesh;
use crate::transform::{Transformable};

fn main() {
    // create a new window application
    let (mut engine, event_loop) = Engine::new("Sinc function", 1000, 800);

    // the scene will store the different objects of the scene
    let mut scene = Scene::new(&engine);

    // a camera is necessary to render the scene to the screen
    let mut camera = Camera::new(&engine);
    
    // the camera control allows the user to move the camera around with the mouse
    let mut camera_control = OrbitControl::default();
    camera_control.set_radius(2.0);
    camera.control = Some(Box::new(camera_control));
    
    // we set the camera as the active camera of the scene
    scene.set_active_camera(camera);

    // then we create a procedural mesh
    let mut mesh = ProceduralMesh::terrain("sinc", 10.0, 128, &|x, z| {
        // simple sinc function
        let d = 5.0 * (x * x + z * z).sqrt();
        3.0 * (f32::sin(d) / d).min(1.0)
    }, 1.0, &engine);
    
    // set the color of the mesh
    mesh.material().set_diffuse_color(0.5, 0.2, 1.0);
    
    // add the mesh to the scene
    scene.add_mesh(mesh);

    // start the engine
    engine.start(scene, event_loop);
}
```

## About

Loosely based
on [Dr. Xu youtube series](https://www.youtube.com/watch?v=i6WMfY-XTZE&list=PL_UrKDEhALdJS0VrLPn7dqC5A4W1vCAUT)
and the [WGPU tutorial](https://sotrh.github.io/learn-wgpu/).