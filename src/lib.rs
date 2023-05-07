use gfx_hal::pso::PrimitiveAssemblerDesc::Mesh;
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

mod engine;
mod scene;
mod camera;
mod mesh;
mod procedural_plane;
mod transform;
mod vertex_data;
mod material;

use crate::engine::Engine;
use crate::scene::Scene;

pub fn init_gilgamesh() -> (EventLoop<()>, Window, Engine, Scene) {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    window.set_title("Gilgamesh");

    let mut engine = pollster::block_on(Engine::init_wgpu(&window));
    let mut scene = Scene::new(&window);

    scene.execute_before_render = Box::new(move || {});

    (event_loop, window, engine, scene)
}

pub fn start_gilgamesh(event_loop: EventLoop<()>, window: Window, mut engine: Engine, mut scene: Scene) {
    let start_time = std::time::Instant::now();

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id
        } if window_id == window.id() => {
            engine.manage_event(event);
            scene.manage_event(event);

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
                _ => {}
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