extern crate gilgamesh;

use cgmath::Rotation3;
use winit::event::VirtualKeyCode::*;
use gilgamesh::camera::BasicCamera;
use gilgamesh::engine::Engine;
use gilgamesh::mesh::Mesh;
use gilgamesh::scene::Scene;
use gilgamesh::transform::Transformable;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn run() {
    let (mut engine, event_loop) = Engine::new("Gilgamesh", 1000, 800);

    let mut scene = Scene::new(&engine);

    let mut camera = BasicCamera::new(&engine);
    camera.transform.set_position(3.0, 1.5, 3.0);

    scene.set_active_camera(camera);

    let sphere = Mesh::new_procedural_sphere("Sphere".into(), 5.0, 32, &|x, y, z| {
        f32::powi(f32::sin(60.0 * x * y * z), 2) * 0.5
    }, 0.5, &mut engine);


    let sphere_idx = scene.add_mesh(sphere);

    scene.on_before_render.push(Box::new(move |engine, active_camera, meshes| {
        meshes[sphere_idx].transform.set_position(0.0, engine.get_elapsed_time().sin(), 0.0);
    }));

    scene.on_mouse_moved.push(Box::new(move |engine, active_camera, mouse_position| {
        println!("Mouse moved to {:?}", mouse_position);
    }));

    scene.on_key_pressed.push(Box::new(|engine, active_camera, key| {
        match key {
            T => {
                println!("T pressed at {}", engine.get_elapsed_time());
            }
            Down => {
                let rotation = cgmath::Quaternion::from_axis_angle(
                    active_camera.as_mut().unwrap().transform.right(),
                    cgmath::Deg(1.0),
                );
                active_camera.as_mut().unwrap().transform.position = rotation * active_camera.as_mut().unwrap().transform.position;
            }
            Right => {
                let rotation = cgmath::Quaternion::from_axis_angle(
                    cgmath::Vector3::unit_y(),
                    cgmath::Deg(1.0),
                );
                active_camera.as_mut().unwrap().transform.position = rotation * active_camera.as_mut().unwrap().transform.position;
            }
            Up => {
                let rotation = cgmath::Quaternion::from_axis_angle(
                    active_camera.as_mut().unwrap().transform.right(),
                    cgmath::Deg(-1.0),
                );
                active_camera.as_mut().unwrap().transform.position = rotation * active_camera.as_mut().unwrap().transform.position;
            }
            Left => {
                let rotation = cgmath::Quaternion::from_axis_angle(
                    cgmath::Vector3::unit_y(),
                    cgmath::Deg(-1.0),
                );
                active_camera.as_mut().unwrap().transform.position = rotation * active_camera.as_mut().unwrap().transform.position;
            }
            _ => {}
        }
    }));

    engine.start(scene, event_loop);
}

fn main() {
    run();
}