use gilgamesh::camera::{Camera, CameraProjection, Perspective};
use gilgamesh::core::app::{App, FrameContext};
use gilgamesh::core::engine::Engine;
use gilgamesh::core::render_context::FrameBindingsData;
use gilgamesh::core::wgpu_context::WGPUContext;
use gilgamesh::geometry::mesh::Mesh;
use gilgamesh::geometry::procedural;
use gilgamesh::input::orbit_control::OrbitControl;
use gilgamesh::lights::directional_light::DirectionalLight;
use gilgamesh::materials::blinn_phong::BlinnPhongMaterial;
use std::rc::Rc;

fn main() {
    Engine::<TerrainView>::run("Sinc function", 1000, 800);
}

struct TerrainView {
    camera: Camera<Perspective>,
    orbit_control: OrbitControl,
    terrain: Mesh,
    directional_light: DirectionalLight,
}

impl App for TerrainView {
    fn new(ctx: &WGPUContext) -> Self {
        let camera = Camera::default();
        let mut orbit_control = OrbitControl::default();
        orbit_control.set_radius(10.0);

        let cpu_buffers = procedural::terrain(10.0, 128, &|x, z| {
            let d = 5.0 * (x * x + z * z).sqrt();
            3.0 * (d.sin() / d).min(1.0)
        });
        let mut material = BlinnPhongMaterial::new("sincMaterial");
        material.set_diffuse_color(0.5, 0.2, 1.0);
        let terrain = Mesh::from_cpu_buffers("sinc", cpu_buffers, Rc::new(material), ctx);

        Self {
            camera,
            orbit_control,
            terrain,
            directional_light: DirectionalLight::default(),
        }
    }

    fn update_before_render(&mut self, frame: &FrameContext<'_>) {
        self.camera.projection.aspect_ratio =
            frame.render_width as f64 / frame.render_height as f64;
        self.orbit_control
            .update(&mut self.camera, frame.mouse, frame.delta_seconds);
    }

    fn get_frame_bindings_data(&mut self) -> FrameBindingsData<'_, impl CameraProjection> {
        FrameBindingsData::new(&self.camera, &self.directional_light, &[])
    }

    fn get_meshes(&mut self) -> impl IntoIterator<Item = &mut Mesh> {
        [&mut self.terrain]
    }
}
