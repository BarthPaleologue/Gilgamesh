use std::mem;
use crate::{Transform, vertex_data};

use bytemuck::{Pod, Zeroable};

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
}

impl Mesh {
    pub fn new() -> Mesh {
        Mesh {
            transform: Transform::new(),
            positions: Vec::new(),
            colors: Vec::new()
        }
    }

    pub fn new_cube() -> Mesh {
        Mesh {
            transform: Transform::new(),
            positions: vertex_data::cube_positions(),
            colors: vertex_data::cube_colors(),
        }
    }
}

pub fn zip_vertex_data(mesh: &Mesh) -> Vec<Vertex> {
    let mut data: Vec<Vertex> = Vec::with_capacity(mesh.positions.len());
    for i in 0..mesh.positions.len() {
        data.push(Vertex {
            position: [mesh.positions[i][0], mesh.positions[i][1], mesh.positions[i][2], 1.0],
            color: [mesh.colors[i][0], mesh.colors[i][1], mesh.colors[i][2], 1.0]
        });
    }
    data.to_vec()
}