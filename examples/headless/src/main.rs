use gilgamesh::blocks::render_scene;
use gilgamesh::camera::{Camera, Perspective};
use gilgamesh::core::render_graph::RenderGraph;
use gilgamesh::core::{App, FrameBindingsData, HeadlessRuntime, WGPUContext};
use gilgamesh::geometry::{Mesh, primitive};
use gilgamesh::lights::DirectionalLight;
use gilgamesh::materials::PbrMaterial;
use gilgamesh::photometry::Nits;
use gilgamesh::transform::Transformable;
use glam::DVec3;
use std::rc::Rc;
use std::time::Duration;

mod checker;
mod output;

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests;

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
    directional_lights: [DirectionalLight; 1],
}

impl App for HeadlessScene {
    fn new(ctx: &WGPUContext) -> Self {
        let camera = Camera::default();
        camera.transform_mut().position = DVec3::new(3.0, 3.0, 5.0);
        camera
            .transform_mut()
            .look_at(DVec3::new(0.0, 0.75, 0.0), DVec3::Y);

        let plane_material = checker::CheckerMaterial {
            colors: [[2_000.0, 3_000.0, 4_000.0], [8_000.0, 9_000.0, 10_000.0]],
            scale: 8.0,
        };
        let plane = Mesh::from_cpu_buffers(
            "Plane",
            &primitive::plane(2, 8.0),
            Rc::new(plane_material),
            ctx,
        );

        let mut sphere_material = PbrMaterial::new("Sphere");
        sphere_material.set_albedo_color(0.9, 0.2, 0.05);
        sphere_material.set_emissive_luminance(Nits::new(10_000.0));
        sphere_material.set_emissive_color(0.03, 0.01, 0.005);
        let sphere = Mesh::from_cpu_buffers(
            "Sphere",
            &primitive::sphere(32),
            Rc::new(sphere_material),
            ctx,
        );
        sphere.transform_mut().position = DVec3::Y;

        Self {
            camera,
            plane,
            sphere,
            directional_lights: [DirectionalLight::default()],
        }
    }

    fn name(&self) -> &'static str {
        "Headless scene"
    }

    fn frame_graph(&self) -> RenderGraph<'_> {
        let mut graph = RenderGraph::new();
        let image = render_scene(
            FrameBindingsData::new(&self.camera, &self.directional_lights, &[]),
            [&self.plane, &self.sphere],
            [3_840.0, 3_840.0, 7_680.0, 1.0],
            &mut graph,
        );
        graph.export(image);
        graph
    }
}
