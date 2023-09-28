use std::mem;
use std::rc::Rc;

use bytemuck::{cast_slice, Pod, Zeroable};
use wgpu::{Buffer, RenderPass};
use wgpu::util::DeviceExt;
use crate::engine::Engine;

use crate::transform::Transform;
use crate::material::Material;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 4],
    pub color: [f32; 4],
    pub normal: [f32; 4],
}

impl Vertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![0=>Float32x4, 1=>Float32x4, 2=>Float32x4];
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
    pub positions: Vec<[f32; 3]>,
    pub colors: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub normals: Vec<[f32; 3]>,
    pub index_buffer: Buffer,
    pub vertex_buffer: Buffer,
    pub material: Rc<Material>,
}

impl Mesh {
    pub fn from_vertex_data(indices: Vec<u32>, positions: Vec<[f32; 3]>, normals: Option<Vec<[f32; 3]>>, engine: &mut Engine) -> Mesh {
        let colors = vec![[0.6, 0.6, 0.6]; positions.len()];
        let normals = match normals {
            Some(v) => v,
            None => create_normals(&positions, &indices)
        };

        let vertex_buffer = engine.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: cast_slice(&zip_vertex_data(&positions, &colors, &normals)),
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
            vertex_buffer,
            indices,
            index_buffer,
            colors,
            normals,
            material: Rc::new(Material::new_default(engine)),
        }
    }

    pub fn draw<'a>(&'a self, render_pass: &mut RenderPass<'a>) {
        self.material.bind(render_pass);

        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.indices.len() as u32, 0, 0..1);
    }
}

pub fn create_normals(positions: &Vec<[f32; 3]>, indices: &Vec<u32>) -> Vec<[f32; 3]> {
    let mut normals = vec![[0.0, 0.0, 0.0]; positions.len()];

    for i in 0..indices.len() / 3 {
        let i0 = indices[i * 3 + 0] as usize;
        let i1 = indices[i * 3 + 1] as usize;
        let i2 = indices[i * 3 + 2] as usize;

        let edge1 = [
            positions[i1][0] - positions[i0][0],
            positions[i1][1] - positions[i0][1],
            positions[i1][2] - positions[i0][2],
        ];
        let edge2 = [
            positions[i2][0] - positions[i0][0],
            positions[i2][1] - positions[i0][1],
            positions[i2][2] - positions[i0][2],
        ];
        let normal = [
            edge1[1] * edge2[2] - edge1[2] * edge2[1],
            edge1[2] * edge2[0] - edge1[0] * edge2[2],
            edge1[0] * edge2[1] - edge1[1] * edge2[0],
        ];

        normals[i0][0] += normal[0];
        normals[i0][1] += normal[1];
        normals[i0][2] += normal[2];

        normals[i1][0] += normal[0];
        normals[i1][1] += normal[1];
        normals[i1][2] += normal[2];

        normals[i2][0] += normal[0];
        normals[i2][1] += normal[1];
        normals[i2][2] += normal[2];
    }

    // Normalize normals
    for normal in normals.iter_mut() {
        let length = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
        normal[0] /= length;
        normal[1] /= length;
        normal[2] /= length;
    }

    normals
}

pub fn zip_vertex_data(positions: &[[f32; 3]], colors: &[[f32; 3]], normals: &[[f32; 3]]) -> Vec<Vertex> {
    let mut data: Vec<Vertex> = Vec::with_capacity(positions.len());
    for i in 0..positions.len() {
        data.push(Vertex {
            position: [positions[i][0], positions[i][1], positions[i][2], 1.0],
            color: [colors[i][0], colors[i][1], colors[i][2], 1.0],
            normal: [normals[i][0], normals[i][1], normals[i][2], 1.0],
        });
    }
    data.to_vec()
}