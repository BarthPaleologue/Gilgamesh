use crate::camera::{Camera, CameraProjection, Perspective};
use crate::core::app::{App, FrameContext};
use crate::core::engine::Engine;
use crate::core::render_context::RenderContext;
use crate::core::wgpu_context::WGPUContext;
use crate::geometry::mesh::Mesh;
use crate::geometry::procedural;
use crate::input::orbit_control::OrbitControl;
use crate::lights::directional_light::DirectionalLight;
use crate::materials::blinn_phong::BlinnPhongMaterial;
use std::rc::Rc;

pub fn run() {
    Engine::run("Sinc function", 1000, 800, TerrainView::new);
}

pub struct TerrainView {
    camera: Camera<Perspective>,
    orbit_control: OrbitControl,
    terrain: Mesh,
    directional_light: DirectionalLight,
}

impl TerrainView {
    pub fn new(ctx: &WGPUContext) -> Self {
        let camera = Camera::default();
        let mut orbit_control = OrbitControl::default();
        orbit_control.set_radius(10.0);

        let cpu_buffers = procedural::terrain(10.0, 128, &|x, z| {
            let d = 5.0 * (x * x + z * z).sqrt();
            3.0 * (d.sin() / d).min(1.0)
        });
        let mut material = BlinnPhongMaterial::new("sincMaterial", ctx);
        material.set_diffuse_color(0.5, 0.2, 1.0);
        let terrain = Mesh::from_cpu_buffers("sinc", cpu_buffers, Rc::new(material), ctx);

        Self {
            camera,
            orbit_control,
            terrain,
            directional_light: DirectionalLight::default(),
        }
    }
}

impl App for TerrainView {
    fn update_before_render(&mut self, frame: &FrameContext<'_>) {
        self.camera.projection.aspect_ratio =
            frame.surface_width as f64 / frame.surface_height as f64;
        self.orbit_control
            .update(&mut self.camera, frame.mouse, frame.delta_seconds);
    }

    fn get_render_context(
        &mut self,
    ) -> RenderContext<'_, impl IntoIterator<Item = &mut Mesh>, impl CameraProjection> {
        RenderContext::new(
            &self.camera,
            [&mut self.terrain],
            &self.directional_light,
            &[],
        )
    }
}
