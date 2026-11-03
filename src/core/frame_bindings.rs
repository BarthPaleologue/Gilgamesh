use crate::camera::Camera;
use crate::camera::uniforms::CameraUniforms;
use crate::core::wgpu_context::WGPUContext;
use crate::lights::directional_light::{DirectionalLight, DirectionalLightUniform};
use crate::lights::point_light::{PointLight, PointLightUniforms};
use crate::materials::utils::{create_array_buffer, create_buffer, uniform_layout_entry};
use crate::settings::MAX_POINT_LIGHTS;

pub(crate) struct FrameBindings {
    pub(crate) layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
    camera: wgpu::Buffer,
    directional_light: wgpu::Buffer,
    point_lights: wgpu::Buffer,
    point_light_count: wgpu::Buffer,
}

impl FrameBindings {
    pub(crate) fn new(ctx: &WGPUContext) -> Self {
        let layout = ctx
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Frame Bind Group Layout"),
                entries: &[
                    uniform_layout_entry(0),
                    uniform_layout_entry(1),
                    uniform_layout_entry(2),
                    uniform_layout_entry(3),
                ],
            });
        let camera = create_buffer::<CameraUniforms>("Camera Buffer", ctx);
        let directional_light =
            create_buffer::<DirectionalLightUniform>("Directional Light Buffer", ctx);
        let point_lights =
            create_array_buffer::<PointLightUniforms>("Point Lights Buffer", MAX_POINT_LIGHTS, ctx);
        let point_light_count = create_buffer::<u32>("Point Light Count Buffer", ctx);
        let entries: Vec<_> = [
            &camera,
            &directional_light,
            &point_lights,
            &point_light_count,
        ]
        .into_iter()
        .enumerate()
        .map(|(binding, buffer)| wgpu::BindGroupEntry {
            binding: binding as u32,
            resource: buffer.as_entire_binding(),
        })
        .collect();
        let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Frame Bind Group"),
            layout: &layout,
            entries: &entries,
        });
        Self {
            layout,
            bind_group,
            camera,
            directional_light,
            point_lights,
            point_light_count,
        }
    }

    pub(crate) fn update(
        &self,
        camera: &Camera,
        directional_light: &DirectionalLight,
        point_lights: &[PointLight],
        queue: &wgpu::Queue,
    ) {
        let mut camera_uniforms = CameraUniforms::default();
        camera_uniforms.update(camera);
        let mut directional_uniforms = DirectionalLightUniform::default();
        directional_uniforms.update(directional_light);
        let mut point_uniforms = [PointLightUniforms::default(); MAX_POINT_LIGHTS];
        for (uniforms, light) in point_uniforms.iter_mut().zip(point_lights) {
            uniforms.update(light);
        }
        queue.write_buffer(&self.camera, 0, bytemuck::bytes_of(&camera_uniforms));
        queue.write_buffer(
            &self.directional_light,
            0,
            bytemuck::bytes_of(&directional_uniforms),
        );
        queue.write_buffer(&self.point_lights, 0, bytemuck::cast_slice(&point_uniforms));
        queue.write_buffer(
            &self.point_light_count,
            0,
            bytemuck::bytes_of(&(point_lights.len() as u32)),
        );
    }

    pub(crate) fn bind(&self, pass: &mut wgpu::RenderPass<'_>) {
        pass.set_bind_group(0, &self.bind_group, &[]);
    }
}
