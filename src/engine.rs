use std::default::Default;
use std::time::SystemTime;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};
use cgmath::*;
use wgpu::{Device, Queue, Surface, SurfaceConfiguration};
use winit::dpi::{PhysicalSize, Size};
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
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

        let (surface, device, queue, config, size) = pollster::block_on(init_wgpu(&window));

        let engine = Engine {
            window,
            surface,
            device,
            queue,
            config,
            size,
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

pub async fn init_wgpu(window: &Window) -> (Surface, Device, Queue, SurfaceConfiguration, PhysicalSize<u32>) {
    let size = window.inner_size();
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        dx12_shader_compiler: Default::default(),
    });

    let surface = unsafe { instance.create_surface(window) }.unwrap();

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

    let surface_caps = surface.get_capabilities(&adapter);
    // Shader code in this tutorial assumes an sRGB surface texture. Using a different
    // one will result all the colors coming out darker. If you want to support non
    // sRGB surfaces, you'll need to account for that when drawing to the frame.
    let surface_format = surface_caps.formats.iter()
        .copied()
        .find(|f| f.is_srgb())
        .unwrap_or(surface_caps.formats[0]);

    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: surface_caps.alpha_modes[0],
        view_formats: vec![],
    };
    surface.configure(&device, &config);

    (surface, device, queue, config, size)
}