use std::iter;
use bytemuck::cast_slice;
use cgmath::{InnerSpace, Rotation3};
use winit::event::{ElementState, KeyboardInput, MouseScrollDelta, VirtualKeyCode, WindowEvent};
use crate::engine::Engine;
use crate::camera::{BasicCamera};
use crate::mesh::{Mesh};

pub const ANIMATION_SPEED: f32 = 1.0;

pub struct Scene {
    pub active_camera: Option<BasicCamera>,
    pub meshes: Vec<Mesh>,
    pub on_key_pressed: Vec<Box<dyn FnMut(&Engine, &VirtualKeyCode)>>,
    pub on_before_render: Vec<Box<dyn FnMut(&Engine, &mut Vec<Mesh>)>>,
}

impl Scene {
    pub fn new(engine: &Engine) -> Scene {
        Scene {
            active_camera: None,
            meshes: Vec::new(),
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
        if new_size.width > 0 && new_size.height > 0 {
            self.active_camera.as_mut().unwrap().aspect_ratio = new_size.width as f32 / new_size.height as f32;
        }
    }

    pub fn manage_event(&mut self, event: &WindowEvent, engine: &Engine) {
        match event {
            WindowEvent::KeyboardInput {
                input:
                KeyboardInput {
                    state: ElementState::Pressed,
                    virtual_keycode: Some(key),
                    ..
                },
                ..
            } => {
                self.on_key_pressed.iter_mut().for_each(|f| f(engine, key));
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
            WindowEvent::KeyboardInput {
                input:
                KeyboardInput {
                    state: ElementState::Pressed,
                    virtual_keycode: Some(VirtualKeyCode::Left),
                    ..
                },
                ..
            } => {
                // rotate camera around the y axis
                let rotation = cgmath::Quaternion::from_axis_angle(
                    cgmath::Vector3::unit_y(),
                    cgmath::Deg(-1.0),
                );
                self.active_camera.as_mut().unwrap().transform.position = rotation * self.active_camera.as_mut().unwrap().transform.position;
            }
            WindowEvent::KeyboardInput {
                input:
                KeyboardInput {
                    state: ElementState::Pressed,
                    virtual_keycode: Some(VirtualKeyCode::Right),
                    ..
                },
                ..
            } => {
                // rotate camera around the y axis
                let rotation = cgmath::Quaternion::from_axis_angle(
                    cgmath::Vector3::unit_y(),
                    cgmath::Deg(1.0),
                );
                self.active_camera.as_mut().unwrap().transform.position = rotation * self.active_camera.as_mut().unwrap().transform.position;
            }
            WindowEvent::KeyboardInput {
                input:
                KeyboardInput {
                    state: ElementState::Pressed,
                    virtual_keycode: Some(VirtualKeyCode::Up),
                    ..
                },
                ..
            } => {
                // rotate camera around the x axis
                let rotation = cgmath::Quaternion::from_axis_angle(
                    self.active_camera.as_mut().unwrap().transform.right(),
                    cgmath::Deg(-1.0),
                );
                self.active_camera.as_mut().unwrap().transform.position = rotation * self.active_camera.as_mut().unwrap().transform.position;
            }
            WindowEvent::KeyboardInput {
                input:
                KeyboardInput {
                    state: ElementState::Pressed,
                    virtual_keycode: Some(VirtualKeyCode::Down),
                    ..
                },
                ..
            } => {
                // rotate camera around the x axis
                let rotation = cgmath::Quaternion::from_axis_angle(
                    self.active_camera.as_mut().unwrap().transform.right(),
                    cgmath::Deg(1.0),
                );
                self.active_camera.as_mut().unwrap().transform.position = rotation * self.active_camera.as_mut().unwrap().transform.position;
            }
            WindowEvent::KeyboardInput {
                input:
                KeyboardInput {
                    state: ElementState::Pressed,
                    virtual_keycode: Some(VirtualKeyCode::W),
                    ..
                },
                ..
            } => {
                //let mesh = self.meshes.first_mut().unwrap();
                //mesh.transform.position += mesh.transform.forward() * ANIMATION_SPEED;
                let camera = &mut self.active_camera.as_mut().unwrap();
                camera.transform.position -= camera.transform.forward() * ANIMATION_SPEED;
            }
            _ => {}
        }
    }

    pub fn render(&mut self, engine: &mut Engine) -> Result<(), wgpu::SurfaceError> {
        self.on_before_render.iter_mut().for_each(|f| f(engine, &mut self.meshes));

        for mesh in self.meshes.iter_mut() {
            let mvp_mat = self.active_camera.as_mut().unwrap().get_projection_matrix() * self.active_camera.as_mut().unwrap().get_view_matrix() * mesh.transform.compute_world_matrix();
            let mvp_ref: &[f32; 16] = mvp_mat.as_ref();
            engine.queue.write_buffer(&mesh.material.vertex_uniform_buffer, 0, cast_slice(mvp_ref));
        }

        //let output = self.init.surface.get_current_frame()?.output;
        let output = engine.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let depth_texture = engine.device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: engine.config.width,
                height: engine.config.height,
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

        let mut encoder = engine.device
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

        engine.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
