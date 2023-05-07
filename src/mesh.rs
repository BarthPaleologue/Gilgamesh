use std::mem;
use std::rc::Rc;

use bytemuck::{cast_slice, Pod, Zeroable};
use wgpu::{Buffer, RenderPass};
use wgpu::util::DeviceExt;

use crate::transform::Transform;
use crate::material::Material;
use crate::engine::Engine;
use crate::vertex_data;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32;4],
    pub color: [f32;4],
}

impl Vertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![0=>Float32x4, 1=>Float32x4];
    pub(crate) fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

pub struct Mesh {
    pub transform: Transform,
    pub positions: Vec<[f32;3]>,
    pub colors: Vec<[f32;3]>,
    pub indices: Option<Vec<u32>>,
    pub index_buffer: Option<Buffer>,
    pub vertex_buffer: Buffer,
    pub material: Rc<Material>,
}

impl Mesh {
    pub fn new(engine: &mut Engine) -> Mesh {
        Mesh {
            transform: Transform::new(),
            indices: None,
            index_buffer: None,
            positions: Vec::new(),
            colors: Vec::new(),
            vertex_buffer: engine.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: &[],
                usage: wgpu::BufferUsages::VERTEX,
            }),
            material: Rc::new(Material::new(engine))
        }
    }

    pub fn from_data(indices: Vec<u32>, positions: Vec<[f32;3]>, engine: &mut Engine) -> Mesh {
        let mut colors = Vec::new();
        for _ in 0..positions.len() {
            colors.push([0.2, 0.2, 0.2]);
        }
        let vertex_buffer = engine.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: cast_slice(&zip_vertex_data(&positions, &colors)),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = engine.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        Mesh {
            transform: Transform::new(),
            positions,
            indices: Some(indices),
            index_buffer: Some(index_buffer),
            colors,
            vertex_buffer,
            material: Rc::new(Material::new(engine))
        }
    }

    pub fn new_cube(engine: &mut Engine) -> Mesh {
        let positions = vertex_data::cube_positions();
        let colors = vertex_data::cube_colors();
        let vertex_buffer = engine.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: cast_slice(&zip_vertex_data(&positions, &colors)),
            usage: wgpu::BufferUsages::VERTEX,
        });
        Mesh {
            transform: Transform::new(),
            positions,
            indices: None,
            index_buffer: None,
            colors,
            vertex_buffer,
            material: Rc::new(Material::new(engine))
        }
    }

    pub fn draw<'a, 'b>(&'a self, render_pass: &'b mut RenderPass<'a>) -> () {
        self.material.bind(render_pass);

        if let Some(index_buffer) = &self.index_buffer {
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..self.indices.as_ref().unwrap().len() as u32, 0, 0..1);
        } else {
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..self.positions.len() as u32, 0..1);
        }
    }
}

pub fn zip_vertex_data(positions: &Vec<[f32;3]>, colors: &Vec<[f32;3]>) -> Vec<Vertex> {
    let mut data: Vec<Vertex> = Vec::with_capacity(positions.len());
    for i in 0..positions.len() {
        data.push(Vertex {
            position: [positions[i][0], positions[i][1], positions[i][2], 1.0],
            color: [colors[i][0], colors[i][1], colors[i][2], 1.0]
        });
    }
    data.to_vec()
}