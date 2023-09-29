use std::mem;
use std::rc::Rc;

use bytemuck::{cast_slice, Pod, Zeroable};
use wgpu::{Buffer, RenderPass};
use wgpu::util::DeviceExt;
use crate::core::engine::Engine;
use crate::geometry::vertex_data::VertexData;

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
    pub name: String,
    pub transform: Transform,
    pub vertex_data: VertexData,
    pub index_buffer: Buffer,
    pub vertex_buffer: Buffer,
    pub material: Rc<Material>,
}

impl Mesh {
    pub fn from_vertex_data(name: String, vertex_data: VertexData, engine: &mut Engine) -> Mesh {
        let colors = vec![[0.6, 0.6, 0.6]; vertex_data.positions.len()];
        let normals = vertex_data.normals;
        let positions = vertex_data.positions;
        let indices = vertex_data.indices;

        let vertex_buffer = engine.wgpu_context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: cast_slice(&zip_vertex_data(&positions, &colors, &normals)),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = engine.wgpu_context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let nb_vertices = positions.len();
        let vertex_data = VertexData {
            positions,
            colors,
            normals,
            indices,
            uvs: vec![[0.0, 0.0]; nb_vertices],
        };

        Mesh {
            name,
            transform: Transform::new(),
            vertex_data,
            vertex_buffer,
            index_buffer,
            material: Rc::new(Material::new_default(&mut engine.wgpu_context)),
        }
    }

    pub fn draw<'a>(&'a self, render_pass: &mut RenderPass<'a>) {
        self.material.bind(render_pass);

        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.vertex_data.indices.len() as u32, 0, 0..1);
    }
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