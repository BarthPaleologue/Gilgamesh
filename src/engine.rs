use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};
use cgmath::*;
use wgpu::{Device, Queue, Surface, SurfaceConfiguration};
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::platform::unix::EventLoopExtUnix;
use crate::scene::Scene;

pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

pub struct Engine {
    pub window: Window,
    pub surface: Surface,
    pub device: Device,
    pub queue: Queue,
    pub config: SurfaceConfiguration,
    pub size: PhysicalSize<u32>,
}


impl Engine {
    pub fn new(name: &str, any_thread: bool) -> (Self, EventLoop<()>) {
        env_logger::init();
        let event_loop = if any_thread { EventLoop::new_any_thread() } else { EventLoop::new() };
        let window = WindowBuilder::new().build(&event_loop).unwrap();
        window.set_title(name);

        let (surface, device, queue, config, size) = pollster::block_on(init_wgpu(&window));

        let app = Engine {
            window,
            surface,
            device,
            queue,
            config,
            size,
        };

        (app, event_loop)
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
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

    pub fn start(mut self, mut scene: Scene, event_loop: EventLoop<()>, mut callback: impl FnMut() + 'static) {
        event_loop.run(move |event, _, control_flow| match event {
            Event::WindowEvent {
                ref event,
                window_id
            } if window_id == self.window.id() => {
                self.manage_event(event);
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
                scene.update(&mut self);

                callback();

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

pub async fn init_wgpu(window: &Window) -> (Surface, Device, Queue, SurfaceConfiguration, PhysicalSize<u32>) {
    let size = window.inner_size();
    let instance = wgpu::Instance::new(wgpu::Backends::all());
    //let instance = wgpu::Instance::new(wgpu::Backends::VULKAN);
    let surface = unsafe { instance.create_surface(window) };
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })
        .await
        .unwrap();

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::POLYGON_MODE_LINE,
                limits: wgpu::Limits::default(),
            },
            None, // Trace path
        )
        .await
        .unwrap();

    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: *surface.get_supported_formats(&adapter).first().unwrap(),
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Fifo,
    };
    surface.configure(&device, &config);

    (surface, device, queue, config, size)
}