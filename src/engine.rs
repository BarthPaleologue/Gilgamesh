use std::default::Default;
use std::time::SystemTime;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};
use cgmath::*;
use wgpu::{Device, Queue, Surface, SurfaceConfiguration};
use winit::dpi::{PhysicalSize, Size};
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use crate::scene::Scene;
use crate::wgpu_context::WGPUContext;

pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

pub struct Engine {
    pub window: Window,
    pub wgpu_context: WGPUContext,

    clock: SystemTime,
    elapsed_time: f32,
    delta_time: f32,

    pub on_window_resize: Vec<Box<dyn FnMut(PhysicalSize<u32>)>>,
}


impl Engine {
    pub fn new(name: &str, width: u32, height: u32) -> (Self, EventLoop<()>) {
        cfg_if::cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                std::panic::set_hook(Box::new(console_error_panic_hook::hook));
                console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
            } else {
                env_logger::init();
            }
        }

        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_inner_size(Size::Physical(PhysicalSize { width, height }))
            .build(&event_loop).unwrap();
        window.set_title(name);

        #[cfg(target_arch = "wasm32")]
        {
            // Winit prevents sizing with CSS, so we have to set
            // the size manually when on web.
            use winit::dpi::PhysicalSize;
            window.set_inner_size(PhysicalSize::new(450, 400));

            use winit::platform::web::WindowExtWebSys;
            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| {
                    let dst = doc.get_element_by_id("wasm-example")?;
                    let canvas = web_sys::Element::from(window.canvas());
                    dst.append_child(&canvas).ok()?;
                    Some(())
                })
                .expect("Couldn't append canvas to document body.");
        }

        let wgpu_context = pollster::block_on(WGPUContext::new(&window));

        let engine = Engine {
            window,
            wgpu_context,
            clock: SystemTime::now(),
            elapsed_time: SystemTime::now().elapsed().unwrap().as_secs_f32(),
            delta_time: 0.0,
            on_window_resize: Vec::new(),
        };

        (engine, event_loop)
    }

    pub fn get_delta_time(&self) -> f32 {
        self.delta_time
    }

    pub fn get_elapsed_time(&self) -> f32 {
        self.elapsed_time
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.wgpu_context.config.width = new_size.width;
            self.wgpu_context.config.height = new_size.height;
            self.wgpu_context.surface.configure(&self.wgpu_context.device, &self.wgpu_context.config);
        }
    }

    pub fn manage_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::Resized(physical_size) => {
                self.resize(*physical_size);
            }
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                self.resize(**new_inner_size);
            }
            _ => {}
        }
    }

    pub fn start(mut self, mut scene: Scene, event_loop: EventLoop<()>) {
        event_loop.run(move |event, _, control_flow| match event {
            Event::WindowEvent {
                ref event,
                window_id
            } if window_id == self.window.id() => {
                self.manage_event(event);
                scene.manage_event(event, &self);

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
                let new_elapsed_time = self.clock.elapsed().unwrap().as_secs_f32();
                self.delta_time = new_elapsed_time - self.elapsed_time;
                self.elapsed_time = new_elapsed_time;

                match scene.render(&mut self) {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => {
                        scene.resize(self.window.inner_size());
                        self.resize(self.window.inner_size());
                    }
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    Err(e) => eprintln!("{}", e)
                }
            }

            Event::MainEventsCleared => {
                self.window.request_redraw();
            }
            _ => {}
        });
    }
}