use crate::core::wgpu_context::WGPUContext;
use crate::geometry::cpu_mesh_buffers::CpuMeshBuffers;
use crate::geometry::cpu_vertex_buffers::CpuVertexBuffers;
use bytemuck::{Pod, Zeroable, cast_slice};
use std::mem;
use wgpu::Buffer;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
    pub normal: [f32; 3],
    pub tangent: [f32; 3],
    pub uv: [f32; 2],
}

impl Vertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 5] = wgpu::vertex_attr_array![0=>Float32x3, 1=>Float32x3, 2=>Float32x3, 3=>Float32x3, 4=>Float32x2];
    pub(crate) fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

/// GPU vertex and index buffers, without a retained copy of the CPU geometry.
#[must_use]
pub struct GpuMeshBuffers {
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    index_count: u32,
}

impl GpuMeshBuffers {
    /// Uses existing GPU buffers with the interleaved `Vertex` layout and `u32` indices.
    /// Buffers must support `VERTEX` and `INDEX` usage respectively; compute outputs
    /// also need `STORAGE` usage.
    pub fn new(vertex_buffer: Buffer, index_buffer: Buffer, index_count: u32) -> Self {
        Self {
            vertex_buffer,
            index_buffer,
            index_count,
        }
    }

    /// Uploads vertex attributes and indices without retaining the CPU data.
    pub fn from_cpu_buffers(cpu_buffers: &CpuMeshBuffers, wgpu_context: &WGPUContext) -> Self {
        let vertex_buffer =
            wgpu_context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: cast_slice(&to_gpu_vertices(&cpu_buffers.vertices)),
                    usage: wgpu::BufferUsages::VERTEX,
                });
        let index_buffer =
            wgpu_context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: cast_slice(&cpu_buffers.indices),
                    usage: wgpu::BufferUsages::INDEX,
                });
        Self::new(
            vertex_buffer,
            index_buffer,
            cpu_buffers.indices.len() as u32,
        )
    }

    #[must_use]
    pub fn vertex_buffer(&self) -> &Buffer {
        &self.vertex_buffer
    }

    #[must_use]
    pub fn index_buffer(&self) -> &Buffer {
        &self.index_buffer
    }

    #[must_use]
    pub fn index_count(&self) -> u32 {
        self.index_count
    }
}

#[must_use]
pub fn to_gpu_vertices(vertices: &CpuVertexBuffers) -> Vec<Vertex> {
    let nb_vertices = vertices.positions.len();
    let mut data: Vec<Vertex> = Vec::with_capacity(nb_vertices);
    for i in 0..nb_vertices {
        data.push(Vertex {
            position: vertices.positions[i],
            color: vertices.colors[i],
            normal: vertices.normals[i],
            tangent: vertices.tangents[i],
            uv: vertices.uvs[i],
        });
    }
    data
}
