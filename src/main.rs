extern crate gilgamesh;

use winit::event::VirtualKeyCode::*;
use gilgamesh::camera::camera::Camera;
use gilgamesh::input::transform_control::OrbitControl;
use gilgamesh::core::engine::Engine;
use gilgamesh::core::scene::Scene;
use gilgamesh::geometry::primitive::PrimitiveMesh;
use gilgamesh::lights::light::Light;
use gilgamesh::lights::point_light::PointLight;
use gilgamesh::transform::{Transform, Transformable};

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn run() {
    let (mut engine, event_loop) = Engine::new("Gilgamesh", 1000, 800);

    let mut scene = Scene::new(&engine);

    let mut camera = Camera::new(&engine);
    camera.control = Some(Box::<OrbitControl>::default());

    scene.set_active_camera(camera);

    let mut point_light = PointLight::default();
    point_light.set_color(1.0, 0.0, 0.0);
    point_light.transform_mut().set_position(0.0, 5.0, 0.0);
    scene.add_point_light(point_light);

    let mut point_light2 = PointLight::default();
    point_light2.set_color(0.0, 0.0, 1.0);
    point_light2.set_intensity(2.0);
    point_light2.transform_mut().set_position(0.0, -5.0, 0.0);
    scene.add_point_light(point_light2);

    let cube1 = PrimitiveMesh::cube("Cube1", &mut engine);
    let cube1_idx = scene.add_mesh(cube1);

    let sphere1 = PrimitiveMesh::sphere("Cube", 32, &mut engine);
    sphere1.transform_mut().parent = Some(scene.meshes[cube1_idx].transform_rc());
    let sphere1_idx = scene.add_mesh(sphere1);

    let sphere2 = PrimitiveMesh::sphere("Sphere2", 32, &mut engine);
    sphere2.transform_mut().set_position(0.0, 0.0, -8.0);
    sphere2.transform_mut().set_scaling(0.7, 0.7, 0.7);
    scene.add_mesh(sphere2);

    let plane = PrimitiveMesh::plane("Plane", 10, 10.0, &mut engine);
    plane.transform_mut().set_position(0.0, -5.0, 0.0);
    let plane_idx = scene.add_mesh(plane);

    scene.on_before_render.push(Box::new(move |engine, active_camera, meshes, mouse| {
        meshes[cube1_idx].transform_mut().set_rotation(
            0.0,
            engine.get_elapsed_time(),
            0.0,
        );

        meshes[sphere1_idx].transform_mut().set_position(
            6.0 + f32::cos(2.0 * engine.get_elapsed_time()),
            0.0,
            0.0,
        );
    }));

    scene.on_key_pressed.push(Box::new(|engine, active_camera, key| {
        match key {
            T => {
                println!("T pressed at {}", engine.get_elapsed_time());
            }
            _ => {}
        }
    }));

    engine.start(scene, event_loop);
}

fn main() {
    run();
}