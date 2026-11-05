use crate::camera::uniforms::CameraUniforms;
use crate::core::uniforms::{create_array_buffer, create_buffer, uniform_layout_entry};
use crate::core::wgpu_context::WGPUContext;
use crate::lights::directional_light::{DirectionalLight, DirectionalLightUniform};
use crate::lights::point_light::{PointLight, PointLightUniforms};
use crate::settings::{MAX_DIRECTIONAL_LIGHTS, MAX_POINT_LIGHTS};

pub(crate) struct FrameBindings {
    pub(crate) layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
    camera: wgpu::Buffer,
    directional_lights: wgpu::Buffer,
    point_lights: wgpu::Buffer,
    light_counts: wgpu::Buffer,
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
        let directional_lights = create_array_buffer::<DirectionalLightUniform>(
            "Directional Lights Buffer",
            MAX_DIRECTIONAL_LIGHTS,
            ctx,
        );
        let point_lights =
            create_array_buffer::<PointLightUniforms>("Point Lights Buffer", MAX_POINT_LIGHTS, ctx);
        // WebGL requires uniform blocks to occupy a multiple of 16 bytes.
        let light_counts = create_buffer::<[u32; 4]>("Light Counts Buffer", ctx);
        let entries: Vec<_> = [&camera, &directional_lights, &point_lights, &light_counts]
            .into_iter()
            .zip(0_u32..)
            .map(|(buffer, binding)| wgpu::BindGroupEntry {
                binding,
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
            directional_lights,
            point_lights,
            light_counts,
        }
    }

    pub(crate) fn update(
        &self,
        camera: &CameraUniforms,
        directional_lights: &[DirectionalLight],
        point_lights: &[PointLight],
        queue: &wgpu::Queue,
    ) {
        let mut directional_uniforms = [DirectionalLightUniform::default(); MAX_DIRECTIONAL_LIGHTS];
        for (uniforms, light) in directional_uniforms.iter_mut().zip(directional_lights) {
            uniforms.update(light);
        }
        let mut point_uniforms = [PointLightUniforms::default(); MAX_POINT_LIGHTS];
        for (uniforms, light) in point_uniforms.iter_mut().zip(point_lights) {
            uniforms.update(light);
        }
        queue.write_buffer(&self.camera, 0, bytemuck::bytes_of(camera));
        queue.write_buffer(
            &self.directional_lights,
            0,
            bytemuck::cast_slice(&directional_uniforms),
        );
        queue.write_buffer(&self.point_lights, 0, bytemuck::cast_slice(&point_uniforms));
        queue.write_buffer(
            &self.light_counts,
            0,
            bytemuck::bytes_of(&[
                u32::try_from(directional_lights.len())
                    .expect("Directional light count fits in u32"),
                u32::try_from(point_lights.len()).expect("Point light count fits in u32"),
                0,
                0,
            ]),
        );
    }

    pub(crate) fn bind(&self, pass: &mut wgpu::RenderPass<'_>) {
        pass.set_bind_group(0, &self.bind_group, &[]);
    }
}
