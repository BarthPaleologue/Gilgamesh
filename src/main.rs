mod engine;
mod vertex_data;
mod transform;
mod camera;
mod mesh;
mod material;
mod scene;

use std::cell::RefCell;
use std::rc::Rc;
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
use crate::scene::{ANIMATION_SPEED, Scene};

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    window.set_title("Bonjour");

    let start_time = std::time::Instant::now();

    let mut engine = pollster::block_on(Engine::init_wgpu(&window));

    let cube = Mesh::new_cube(&mut engine);
    let cube_rc = Rc::new(cube);

    let mut cube2 = Mesh::new_cube(&mut engine);
    cube2.transform.position.y = 2.5;

    let cube2_rc = Rc::new(cube2);

    {
        let mut scene = Scene::new(&window);

        scene.meshes.push(Rc::clone(&cube_rc));
        scene.meshes.push(Rc::clone(&cube2_rc));

        let update_meshes = move || {
            //let dt = (std::time::Instant::now() - start_time).as_secs_f32();
            //cube2.transform.rotation.y += ANIMATION_SPEED;
        };

        scene.execute_before_render = Box::new(update_meshes);

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
                            scene.resize(*physical_size);
                            engine.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            scene.resize(**new_inner_size);
                            engine.resize(**new_inner_size);
                        }
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(_) => {
                let now = std::time::Instant::now();
                let dt = now - start_time;
                scene.update(&mut engine, dt);

                match scene.render(&mut engine) {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => {
                        scene.resize(window.inner_size());
                        engine.resize(window.inner_size());
                    }
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
}