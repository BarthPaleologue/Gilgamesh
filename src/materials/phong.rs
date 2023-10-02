use crate::core::wgpu_context::WGPUContext;
use crate::materials::material_pipeline::MaterialPipeline;

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
}

impl PhongMaterial {
    fn new(wgpu_context: &mut WGPUContext) -> Self {
        PhongMaterial {
            phong_uniforms: PhongUniforms::default(),
            material_pipeline: MaterialPipeline::new_default(wgpu_context),
        }
    }
}