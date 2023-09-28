extern crate gilgamesh;

use cgmath::Rotation3;
use winit::event::VirtualKeyCode;
use gilgamesh::engine::Engine;
use gilgamesh::mesh::Mesh;
use gilgamesh::scene::Scene;

fn main() {
    let (mut engine, event_loop) = Engine::new("Gilgamesh", false);

    let mut scene = Scene::new(&engine);

    let sphere = Mesh::new_procedural_sphere("Sphere".into(), 5.0, 32, &|x, y, z| {
        f32::powi(f32::sin(60.0 * x * y * z), 2) * 0.5
    }, 0.5, &mut engine);


    scene.add_mesh(sphere);

    scene.on_before_render.push(Box::new(|engine, meshes| {
        meshes.get_mut("Sphere").unwrap().transform.set_position(0.0, engine.get_elapsed_time().sin(), 0.0);
    }));

    scene.on_key_pressed.push(Box::new(|engine, key| {
        match key {
            &VirtualKeyCode::T => {
                println!("T pressed at {}", engine.get_elapsed_time());
            }
            &VirtualKeyCode::Left => {
                let rotation = cgmath::Quaternion::from_axis_angle(
                    cgmath::Vector3::unit_y(),
                    cgmath::Deg(-1.0),
                );
                //scene.active_camera.transform.position = rotation * scene.active_camera.transform.position;
            }
            _ => {}
        }
    }));

    engine.start(scene, event_loop);
}