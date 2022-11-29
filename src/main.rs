mod engine;
mod vertex_data;
mod transform;
mod camera;
mod mesh;
mod material;
mod scene;

use camera::*;
use engine::Engine;
use transform::{Transform};

use winit::{
    event::*,
    event_loop::{EventLoop, ControlFlow},
    window::{WindowBuilder},
};
use cgmath::num_traits::ToPrimitive;

use crate::mesh::{Vertex, Mesh};
use crate::scene::Scene;

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    window.set_title("Bonjour");

    let start_time = std::time::Instant::now();

    let engine = pollster::block_on(Engine::init_wgpu(&window));

    let mut scene = Scene::new(engine, &window);

    let cube = Mesh::new_cube(&scene.engine);

    let mut cube2 = Mesh::new_cube(&scene.engine);
    cube2.transform.position.y = 2.5;

    scene.meshes.push(cube);
    scene.meshes.push(cube2);

    /*fn update_meshes(scene: &mut Scene) -> () {
        let dt = std::time::Instant::now().as_secs_f32();
        for mut mesh in &mut scene.meshes {
            mesh.transform.rotation.y = ANIMATION_SPEED * dt;
        }
    }*/

    //scene.execute_before_render = update_meshes;

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id
        } if window_id == window.id() => {
            if !scene.input(event) {
                match event {
                    WindowEvent::CloseRequested | WindowEvent::KeyboardInput {
                        input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => {
                        scene.resize(*physical_size)
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        scene.resize(**new_inner_size);
                    }
                    _ => {}
                }
            }
        }
        Event::RedrawRequested(_) => {
            let now = std::time::Instant::now();
            let dt = now - start_time;
            scene.update(dt);

            match scene.render() {
                Ok(_) => {}
                Err(wgpu::SurfaceError::Lost) => scene.resize(scene.engine.size),
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                Err(e) => eprintln!("{}", e)
            }
        }
        Event::MainEventsCleared => {
            window.request_redraw();
        }
        _ => {}
    });
}