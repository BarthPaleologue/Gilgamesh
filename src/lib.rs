use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::platform::unix::EventLoopExtUnix;
use winit::window::{WindowBuilder};
use crate::app::App;

pub mod engine;
pub mod scene;
pub mod camera;
pub mod mesh;
pub mod procedural_plane;
pub mod transform;
pub mod vertex_data;
pub mod material;
pub mod app;

use crate::engine::Engine;
use crate::scene::Scene;

pub fn init_gilgamesh() -> App {
    env_logger::init();
    let event_loop = EventLoop::new_any_thread();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    window.set_title("Gilgamesh");

    let engine = pollster::block_on(Engine::init_wgpu(&window));
    let mut scene = Scene::new(&window);

    scene.execute_before_render = Box::new(move || {});

    App {
        event_loop,
        window,
        engine,
        scene,
    }
}

pub fn start_gilgamesh(app: App) {
    let event_loop = app.event_loop;
    let window = app.window;
    let mut engine = app.engine;
    let mut scene = app.scene;

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
            scene.update(&mut engine);

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