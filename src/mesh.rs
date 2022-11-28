use std::mem;
use crate::{Engine, Transform, vertex_data};

use bytemuck::{cast_slice, Pod, Zeroable};
use wgpu::Buffer;
use wgpu::util::DeviceExt;

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
    pub vertex_buffer: Buffer,
}

impl Mesh {
    pub fn new(engine: &Engine) -> Mesh {
        Mesh {
            transform: Transform::new(),
            positions: Vec::new(),
            colors: Vec::new(),
            vertex_buffer: engine.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: &[],
                usage: wgpu::BufferUsages::VERTEX,
            })
        }
    }

    pub fn new_cube(engine: &Engine) -> Mesh {
        let positions = vertex_data::cube_positions();
        let colors = vertex_data::cube_colors();
        let vertex_buffer = engine.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: cast_slice(&zip_vertex_data(&positions, &colors)),
            usage: wgpu::BufferUsages::VERTEX,
        });
        Mesh {
            transform: Transform::new(),
            positions: positions.clone(),
            colors: colors.clone(),
            vertex_buffer
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