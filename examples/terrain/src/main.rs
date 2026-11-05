use gilgamesh::blocks::render_scene;
use gilgamesh::camera::{Camera, Perspective};
use gilgamesh::core::render_graph::RenderGraph;
use gilgamesh::core::{App, FrameBindingsData, FrameContext, WGPUContext, WindowRuntime};
use gilgamesh::geometry::{Mesh, procedural};
use gilgamesh::input::OrbitControl;
use gilgamesh::lights::DirectionalLight;
use gilgamesh::materials::PbrMaterial;
use std::rc::Rc;

fn main() {
    WindowRuntime::<TerrainView>::run(1000, 800);
}

struct TerrainView {
    camera: Camera<Perspective>,
    orbit_control: OrbitControl,
    terrain: Mesh,
    directional_lights: [DirectionalLight; 1],
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
        let mut material = PbrMaterial::new("sincMaterial");
        material.set_albedo_color(0.5, 0.2, 1.0);
        let terrain = Mesh::from_cpu_buffers("sinc", &cpu_buffers, Rc::new(material), ctx);

        Self {
            camera,
            orbit_control,
            terrain,
            directional_lights: [DirectionalLight::default()],
        }
    }

    fn name(&self) -> &'static str {
        "Sinc function"
    }

    fn update_before_frame(&mut self, frame: &FrameContext<'_>) {
        self.orbit_control
            .update(&mut self.camera, frame.mouse, frame.delta_seconds);
    }

    fn frame_graph(&self) -> RenderGraph<'_> {
        let mut graph = RenderGraph::new();
        let image = render_scene(
            FrameBindingsData::new(&self.camera, &self.directional_lights, &[]),
            [&self.terrain],
            [3_840.0, 3_840.0, 7_680.0, 1.0],
            &mut graph,
        );
        graph.export(image);
        graph
    }
}
