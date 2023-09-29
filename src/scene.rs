use std::iter;
use bytemuck::cast_slice;
use cgmath::{InnerSpace};
use winit::event::{ElementState, KeyboardInput, MouseScrollDelta, VirtualKeyCode, WindowEvent};
use crate::engine::Engine;
use crate::camera::{BasicCamera};
use crate::mesh::{Mesh};
use crate::mouse::Mouse;

pub const ANIMATION_SPEED: f32 = 1.0;

pub type SceneClosure = Box<dyn FnMut(&Engine, &mut Option<BasicCamera>, &mut Vec<Mesh>, &Mouse)>;

pub struct Scene {
    pub active_camera: Option<BasicCamera>,
    pub meshes: Vec<Mesh>,
    pub mouse: Mouse,
    pub on_key_pressed: Vec<Box<dyn FnMut(&Engine, &mut Option<BasicCamera>, &VirtualKeyCode)>>,
    pub on_before_render: Vec<SceneClosure>,
}

impl Scene {
    pub fn new(engine: &Engine) -> Scene {
        Scene {
            active_camera: None,
            meshes: Vec::new(),
            mouse: Mouse::new(),
            on_key_pressed: Vec::new(),
            on_before_render: Vec::new(),
        }
    }

    pub fn set_active_camera(&mut self, camera: BasicCamera) {
        self.active_camera = Some(camera);
    }

    pub fn add_mesh(&mut self, mesh: Mesh) -> usize {
        self.meshes.push(mesh);
        self.meshes.len() - 1
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.height > 0 {
            self.active_camera.as_mut().unwrap().aspect_ratio = new_size.width as f32 / new_size.height as f32;
        }
    }

    pub fn manage_event(&mut self, event: &WindowEvent, engine: &Engine) {
        self.mouse.listen_to_event(event);

        match event {
            WindowEvent::KeyboardInput {
                input: KeyboardInput {
                    state: ElementState::Pressed,
                    virtual_keycode: Some(key),
                    ..
                },
                ..
            } => {
                self.on_key_pressed.iter_mut().for_each(|f| f(engine, &mut self.active_camera, key));
            }
            _ => {}
        }

        match event {
            WindowEvent::Resized(physical_size) => {
                self.resize(*physical_size);
            }
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                self.resize(**new_inner_size);
            }
            WindowEvent::MouseWheel {
                delta: MouseScrollDelta::LineDelta(_, y),
                ..
            } => {
                let out_dir = self.active_camera.as_mut().unwrap().transform.position.normalize();
                self.active_camera.as_mut().unwrap().transform.position -= out_dir * *y * 0.1;
            }
            _ => {}
        }
    }

    pub fn render(&mut self, engine: &mut Engine) -> Result<(), wgpu::SurfaceError> {
        if let Some(active_camera) = &mut self.active_camera {
            active_camera.listen_to_control(&self.mouse);
        }

        self.on_before_render.iter_mut().for_each(|f| f(engine, &mut self.active_camera, &mut self.meshes, &self.mouse));

        for mesh in self.meshes.iter_mut() {
            let mvp_mat = self.active_camera.as_mut().unwrap().get_projection_matrix() * self.active_camera.as_mut().unwrap().get_view_matrix() * mesh.transform.compute_world_matrix();
            let mvp_ref: &[f32; 16] = mvp_mat.as_ref();
            engine.wgpu_context.queue.write_buffer(&mesh.material.vertex_uniform_buffer, 0, cast_slice(mvp_ref));
        }

        //let output = self.init.surface.get_current_frame()?.output;
        let output = engine.wgpu_context.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let depth_texture = engine.wgpu_context.device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: engine.wgpu_context.config.width,
                height: engine.wgpu_context.config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth24Plus,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: None,
            view_formats: &[],
        });
        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = engine.wgpu_context.device
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
                            r: 0.1,
                            g: 0.1,
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

            for mesh in self.meshes.iter_mut() {
                mesh.draw(&mut render_pass);
            }
        }

        engine.wgpu_context.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
