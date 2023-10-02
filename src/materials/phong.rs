use std::cell::Ref;
use std::ops::Deref;
use bytemuck::cast_slice;
use wgpu::RenderPass;
use crate::camera::camera::Camera;
use crate::core::wgpu_context::WGPUContext;
use crate::lights::point_light::PointLight;
use crate::materials::material_pipeline::MaterialPipeline;
use crate::transform::Transform;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PhongUniforms {
    diffuse_color: [f32; 3],
    _padding1: u32,
    ambient_color: [f32; 3],
    _padding2: u32,
    specular_color: [f32; 3],
    _padding3: u32,
}

impl Default for PhongUniforms {
    fn default() -> Self {
        PhongUniforms {
            diffuse_color: [1.0, 1.0, 1.0],
            _padding1: 0,
            ambient_color: [0.0, 0.0, 0.0],
            _padding2: 0,
            specular_color: [0.0, 0.0, 0.0],
            _padding3: 0,
        }
    }
}

pub struct PhongMaterial {
    pub phong_uniforms: PhongUniforms,
    material_pipeline: MaterialPipeline,

    //point_lights_uniform_buffer: wgpu::Buffer,
    //nb_point_lights_buffer: wgpu::Buffer,
}

impl PhongMaterial {
    fn new(wgpu_context: &mut WGPUContext) -> Self {
        let material_pipeline = MaterialPipeline::new_default(wgpu_context);
        PhongMaterial {
            phong_uniforms: PhongUniforms::default(),
            material_pipeline,
        }
    }

    /*pub fn bind<'a, 'b>(&'a mut self, render_pass: &'b mut RenderPass<'a>, transform: Ref<Transform>, active_camera: &Camera, point_lights: &[PointLight], directional_light: &DirectionalLight, wgpu_context: &mut WGPUContext) {
        self.transform_uniforms.update(transform.deref());
        wgpu_context.queue.write_buffer(&self.transform_uniforms_buffer, 0, cast_slice(&[self.transform_uniforms]));

        self.camera_uniforms.update(active_camera);
        wgpu_context.queue.write_buffer(&self.camera_uniforms_buffer, 0, cast_slice(&[self.camera_uniforms]));

        self.light_uniforms.update(directional_light);
        wgpu_context.queue.write_buffer(&self.light_uniforms_buffer, 0, cast_slice(&[self.light_uniforms]));

        for i in 0..point_lights.len() {
            self.point_light_uniforms[i].update(&point_lights[i]);
        }
        wgpu_context.queue.write_buffer(&self.point_light_buffer, 0, cast_slice(&[self.point_light_uniforms]));
        wgpu_context.queue.write_buffer(&self.nb_point_lights_buffer, 0, cast_slice(&[point_lights.len() as u32]));

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.transform_bind_group, &[]);
        render_pass.set_bind_group(1, &self.uniform_bind_group, &[]);
    }*/
}