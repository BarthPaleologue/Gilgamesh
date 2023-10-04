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
use crate::materials::phong::PhongMaterial;

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
    pub(crate) fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

pub struct Mesh {
    name: String,
    transform: Rc<RefCell<Transform>>,
    vertex_data: VertexData,
    index_buffer: Buffer,
    vertex_buffer: Buffer,
    material: PhongMaterial,
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
    pub fn from_vertex_data(name: &str, vertex_data: VertexData, engine: &Engine) -> Mesh {
        let vertex_buffer = engine.wgpu_context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: cast_slice(&zip_vertex_data(&vertex_data)),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = engine.wgpu_context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: cast_slice(&vertex_data.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        Mesh {
            name: name.to_string(),
            transform: Rc::new(RefCell::new(Transform::new())),
            vertex_data,
            vertex_buffer,
            index_buffer,
            material: PhongMaterial::new(&engine.wgpu_context),
        }
    }

    pub fn material(&mut self) -> &mut PhongMaterial {
        &mut self.material
    }

    pub fn set_material(&mut self, material: PhongMaterial) {
        self.material = material;
    }

    /// you may be asking wtf is going on with the lifetimes here, and I don't know either. Dark magic.
    pub fn render<'a, 'b>(&'a mut self, render_pass: &'b mut RenderPass<'a>, active_camera: &Camera, directional_light: &DirectionalLight, point_lights: &[PointLight], wgpu_context: &WGPUContext) {
        let transform = self.transform_rc();
        self.material.bind(render_pass, transform.borrow(), active_camera, point_lights, directional_light, wgpu_context);

        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.vertex_data.indices.len() as u32, 0, 0..1);
    }
}

pub fn zip_vertex_data(vertex_data: &VertexData) -> Vec<Vertex> {
    let nb_vertices = vertex_data.positions.len();
    let mut data: Vec<Vertex> = Vec::with_capacity(nb_vertices);
    for i in 0..nb_vertices {
        data.push(Vertex {
            position: vertex_data.positions[i],
            color: vertex_data.colors[i],
            normal: vertex_data.normals[i],
            tangent: vertex_data.tangents[i],
            uv: vertex_data.uvs[i],
        });
    }
    data.to_vec()
}