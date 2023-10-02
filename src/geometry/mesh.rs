use std::cell::{Ref, RefCell, RefMut};
use std::mem;
use std::rc::Rc;

use bytemuck::{cast_slice, Pod, Zeroable};
use wgpu::{Buffer, RenderPass};
use wgpu::util::DeviceExt;
use crate::camera::camera::Camera;
use crate::core::engine::Engine;
use crate::core::wgpu_context::WGPUContext;
use crate::geometry::vertex_data::VertexData;
use crate::lights::directional_light::DirectionalLight;
use crate::lights::point_light::PointLight;

use crate::transform::{Transform, Transformable};
use crate::materials::material_pipeline::MaterialPipeline;
use crate::materials::phong::PhongMaterial;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

impl Vertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 4] = wgpu::vertex_attr_array![0=>Float32x3, 1=>Float32x3, 2=>Float32x3, 3=>Float32x2];
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
    transform: Rc<RefCell<Transform>>,
    pub vertex_data: VertexData,
    pub index_buffer: Buffer,
    pub vertex_buffer: Buffer,
    pub material: PhongMaterial,
}

impl Transformable for Mesh {
    fn transform(&self) -> Ref<Transform> {
        self.transform.borrow()
    }
    fn transform_mut(&self) -> RefMut<Transform> {
        self.transform.borrow_mut()
    }
    fn transform_rc(&self) -> Rc<RefCell<Transform>> {
        self.transform.clone()
    }
}

impl Mesh {
    pub fn from_vertex_data(name: &str, vertex_data: VertexData, engine: &mut Engine) -> Mesh {
        let colors = vec![[0.6, 0.6, 0.6]; vertex_data.positions.len()];
        let normals = vertex_data.normals;
        let positions = vertex_data.positions;
        let indices = vertex_data.indices;
        let uvs = vertex_data.uvs;

        let vertex_buffer = engine.wgpu_context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: cast_slice(&zip_vertex_data(&positions, &colors, &normals, &uvs)),
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
            name: name.to_string(),
            transform: Rc::new(RefCell::new(Transform::new())),
            vertex_data,
            vertex_buffer,
            index_buffer,
            material: PhongMaterial::new(&mut engine.wgpu_context),
        }
    }

    /// you may be asking wtf is going on with the lifetimes here, and I don't know either. Dark magic.
    pub fn render<'a, 'b>(&'a mut self, render_pass: &'b mut RenderPass<'a>, active_camera: &Camera, directional_light: &DirectionalLight, point_lights: &[PointLight], wgpu_context: &mut WGPUContext) {
        let transform = self.transform_rc();
        self.material.material_pipeline.bind(render_pass, transform.borrow(), active_camera, point_lights, directional_light, wgpu_context);

        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.vertex_data.indices.len() as u32, 0, 0..1);
    }
}

pub fn zip_vertex_data(positions: &[[f32; 3]], colors: &[[f32; 3]], normals: &[[f32; 3]], uvs: &[[f32; 2]]) -> Vec<Vertex> {
    let mut data: Vec<Vertex> = Vec::with_capacity(positions.len());
    for i in 0..positions.len() {
        data.push(Vertex {
            position: [positions[i][0], positions[i][1], positions[i][2]],
            color: [colors[i][0], colors[i][1], colors[i][2]],
            normal: [normals[i][0], normals[i][1], normals[i][2]],
            uv: [uvs[i][0], uvs[i][1]],
        });
    }
    data.to_vec()
}