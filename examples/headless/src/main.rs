use gilgamesh::blocks::{rasterize, tone_map};
use gilgamesh::camera::{Camera, Perspective};
use gilgamesh::core::app::App;
use gilgamesh::core::headless_runtime::HeadlessRuntime;
use gilgamesh::core::render_context::FrameBindingsData;
use gilgamesh::core::render_graph::{RenderGraph, TextureFormat};
use gilgamesh::core::wgpu_context::WGPUContext;
use gilgamesh::geometry::{mesh::Mesh, primitive};
use gilgamesh::lights::directional_light::DirectionalLight;
use gilgamesh::materials::blinn_phong::BlinnPhongMaterial;
use gilgamesh::transform::Transformable;
use glam::DVec3;
use std::rc::Rc;
use std::time::Duration;

mod output;

fn main() {
    HeadlessRuntime::<HeadlessScene>::run(800, 600, async |mut runtime| {
        runtime.render(Duration::ZERO);
        let screenshot = runtime
            .capture_frame()
            .await
            .expect("Failed to capture the headless frame");
        output::save(&screenshot, "headless.png");
    });
}

struct HeadlessScene {
    camera: Camera<Perspective>,
    plane: Mesh,
    sphere: Mesh,
    directional_light: DirectionalLight,
}

impl App for HeadlessScene {
    fn new(ctx: &WGPUContext) -> Self {
        let camera = Camera::default();
        camera.transform_mut().position = DVec3::new(3.0, 3.0, 5.0);
        camera
            .transform_mut()
            .look_at(DVec3::new(0.0, 0.75, 0.0), DVec3::Y);

        let mut plane_material = BlinnPhongMaterial::new("Ground");
        plane_material.set_diffuse_color(0.3, 0.35, 0.4);
        plane_material.set_ambient_color(0.02, 0.02, 0.02);
        let plane = Mesh::from_cpu_buffers(
            "Plane",
            &primitive::plane(2, 8.0),
            Rc::new(plane_material.into()),
            ctx,
        );

        let mut sphere_material = BlinnPhongMaterial::new("Sphere");
        sphere_material.set_diffuse_color(0.9, 0.2, 0.05);
        sphere_material.set_ambient_color(0.03, 0.01, 0.005);
        let sphere = Mesh::from_cpu_buffers(
            "Sphere",
            &primitive::sphere(32),
            Rc::new(sphere_material.into()),
            ctx,
        );
        sphere.transform_mut().position = DVec3::Y;

        Self {
            camera,
            plane,
            sphere,
            directional_light: DirectionalLight::default(),
        }
    }

    fn name(&self) -> &'static str {
        "Headless scene"
    }

    fn frame_graph(&self) -> RenderGraph<'_> {
        let mut graph = RenderGraph::new();
        let scene = rasterize(
            FrameBindingsData::new(&self.camera, &self.directional_light, &[]),
            [&self.plane, &self.sphere],
            TextureFormat::Rgba16Float,
            &mut graph,
        );
        let image = tone_map(scene.color, TextureFormat::Rgba8UnormSrgb, &mut graph);
        graph.export(image);
        graph
    }
}
