use crate::camera::Camera;
use crate::core::app::{App, FrameContext};
use crate::core::engine::Engine;
use crate::core::render_context::RenderContext;
use crate::core::wgpu_context::WGPUContext;
use crate::geometry::mesh::Mesh;
use crate::geometry::procedural;
use crate::input::transform_control::OrbitControl;
use crate::lights::directional_light::DirectionalLight;
use crate::materials::blinn_phong::BlinnPhongMaterial;

pub fn run() {
    Engine::run("Sinc function", 1000, 800, TerrainView::new);
}

pub struct TerrainView {
    camera: Camera,
    terrain: Mesh,
    directional_light: DirectionalLight,
}

impl TerrainView {
    pub fn new(ctx: &WGPUContext) -> Self {
        let mut camera = Camera::new(ctx.config.width as f64 / ctx.config.height as f64);
        let mut camera_control = OrbitControl::default();
        camera_control.set_radius(10.0);
        camera.control = Some(Box::new(camera_control));

        let cpu_buffers = procedural::terrain(10.0, 128, &|x, z| {
            let d = 5.0 * (x * x + z * z).sqrt();
            3.0 * (d.sin() / d).min(1.0)
        });
        let mut terrain = Mesh::from_cpu_buffers("sinc", cpu_buffers, ctx);
        let mut material = BlinnPhongMaterial::new("sincMaterial", ctx);
        material.set_diffuse_color(0.5, 0.2, 1.0);
        terrain.set_material(Box::new(material));

        Self {
            camera,
            terrain,
            directional_light: DirectionalLight::default(),
        }
    }
}

impl App for TerrainView {
    fn update_before_render(&mut self, frame: &FrameContext<'_>) {
        self.camera
            .listen_to_control(frame.mouse, frame.delta_seconds);
    }

    fn get_render_context(&mut self) -> RenderContext<'_, impl IntoIterator<Item = &mut Mesh>> {
        RenderContext::new(
            &mut self.camera,
            [&mut self.terrain],
            &self.directional_light,
            &[],
        )
    }
}
