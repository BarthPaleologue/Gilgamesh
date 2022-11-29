mod engine;
mod vertex_data;
mod transform;
mod camera;
mod mesh;
mod material;

use std::{iter};

use camera::*;
use engine::Engine;
use transform::{Transform};

use winit::{
    event::*,
    event_loop::{EventLoop, ControlFlow},
    window::{Window, WindowBuilder},
};
use bytemuck::{cast_slice};
use cgmath::num_traits::ToPrimitive;

use crate::mesh::{Vertex, Mesh};

const ANIMATION_SPEED: f32 = 1.0;

struct Scene {
    engine: Engine,
    basic_camera: BasicCamera,
    meshes: Vec<Mesh>
}

impl Scene {
    fn new(engine: Engine, window: &Window) -> Scene {
        let mut free_camera = FreeCamera::new(window.inner_size().width as f32 / window.inner_size().height as f32);

        free_camera.tf().set_position(3.0, 1.5, 3.0);

        let cube = Mesh::new_cube(&engine);

        let mut meshes: Vec<Mesh> = Vec::new();
        meshes.push(cube);

        Scene {
            engine,
            basic_camera: free_camera.basic_camera,
            meshes
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.engine.size = new_size;
            self.engine.config.width = new_size.width;
            self.engine.config.height = new_size.height;
            self.engine.surface.configure(&self.engine.device, &self.engine.config);

            self.basic_camera.aspect_ratio = new_size.width as f32 / new_size.height as f32;
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    fn update(&mut self, dt: std::time::Duration) {
        let dt = dt.as_secs_f32();
        for mut mesh in &mut self.meshes {
            mesh.transform.rotation.y = ANIMATION_SPEED * dt;
            let mvp_mat = self.basic_camera.get_projection_matrix() * self.basic_camera.get_view_matrix() * mesh.transform.compute_world_matrix();
            let mvp_ref: &[f32; 16] = mvp_mat.as_ref();
            self.engine.queue.write_buffer(&mesh.material.uniform_buffer, 0, cast_slice(mvp_ref));
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        //let output = self.init.surface.get_current_frame()?.output;
        let output = self.engine.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let depth_texture = self.engine.device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: self.engine.config.width,
                height: self.engine.config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth24Plus,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: None,
        });
        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .engine.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.2,
                            g: 0.2,
                            b: 0.2,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                //depth_stencil_attachment: None,
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: false,
                    }),
                    stencil_ops: None,
                }),
            });

            for mesh in &self.meshes {
                mesh.draw(&mut render_pass);
            }
        }

        self.engine.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    window.set_title("Bonjour");

    let start_time = std::time::Instant::now();

    let engine = pollster::block_on(Engine::init_wgpu(&window));

    let mut scene = Scene::new(engine, &window);

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