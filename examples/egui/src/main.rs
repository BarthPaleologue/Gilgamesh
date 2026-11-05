use gilgamesh::blocks::render_scene;
use gilgamesh::camera::{Camera, Perspective};
use gilgamesh::core::render_graph::RenderGraph;
use gilgamesh::core::{
    App, FrameBindingsData, FrameContext, WGPUContext, WindowContext, WindowRuntime,
};
use gilgamesh::geometry::{Mesh, primitive};
use gilgamesh::input::OrbitControl;
use gilgamesh::lights::DirectionalLight;
use gilgamesh::materials::PbrMaterial;
use gilgamesh::photometry::Lux;
use gilgamesh::transform::Transformable;
use gilgamesh_egui::{Egui, egui, egui_overlay};
use glam::DQuat;
use std::rc::Rc;
use winit::event::WindowEvent;

fn main() {
    WindowRuntime::<EguiDemo>::run(1_000, 700);
}

struct EguiDemo {
    camera: Camera<Perspective>,
    orbit: OrbitControl,
    object: Mesh,
    material: Rc<PbrMaterial>,
    lights: [DirectionalLight; 1],
    gui: Egui,
    animate: bool,
}

impl App for EguiDemo {
    fn new(gpu: &WGPUContext) -> Self {
        let mut material = PbrMaterial::new("Object");
        material.set_albedo_color(0.8, 0.25, 0.08);
        let material = Rc::new(material);
        let object = Mesh::from_cpu_buffers("Object", &primitive::cube(), material.clone(), gpu);
        let mut orbit = OrbitControl::default();
        orbit.set_radius(4.0);
        Self {
            camera: Camera::default(),
            orbit,
            object,
            material,
            lights: [DirectionalLight::default()],
            gui: Egui::default(),
            animate: true,
        }
    }

    fn name(&self) -> &'static str {
        "Gilgamesh + egui"
    }

    fn update_before_frame(&mut self, frame: &FrameContext<'_>) {
        let mut material = self.material.as_ref().clone();
        let mut material_changed = false;
        let mut compensation = self.camera.exposure.compensation_stops();
        let mut illuminance = self.lights[0].illuminance().value();
        self.gui.run_window("Scene controls", |ui| {
            ui.label("Drag outside this panel to orbit. Scroll to zoom.");
            ui.separator();
            ui.label("PBR material");
            material_changed |= ui
                .color_edit_button_rgb(&mut material.albedo_color)
                .changed();
            material_changed |= ui
                .add(egui::Slider::new(&mut material.roughness, 0.04..=1.0).text("Roughness"))
                .changed();
            material_changed |= ui
                .add(egui::Slider::new(&mut material.metallic, 0.0..=1.0).text("Metallic"))
                .changed();
            ui.separator();
            ui.add(egui::Slider::new(&mut illuminance, 0.0..=120_000.0).text("Light (lux)"));
            ui.add(
                egui::Slider::new(&mut compensation, -4.0..=4.0)
                    .text("Exposure compensation (stops)"),
            );
            ui.checkbox(&mut self.animate, "Rotate object");
        });
        if material_changed {
            self.material = Rc::new(material);
            self.object.set_material(self.material.clone());
        }
        self.lights[0].set_illuminance(Lux::new(illuminance));
        self.camera.exposure = self.camera.exposure.with_compensation_stops(compensation);
        self.orbit
            .update(&mut self.camera, frame.mouse, frame.delta_seconds);
        if self.animate {
            self.object.transform_mut().rotation *=
                DQuat::from_rotation_y(frame.delta_seconds * 0.4);
        }
    }

    fn frame_graph(&self) -> RenderGraph<'_> {
        let mut graph = RenderGraph::new();
        let background = render_scene(
            FrameBindingsData::new(&self.camera, &self.lights, &[]),
            [&self.object],
            [2_000.0, 2_500.0, 4_000.0, 1.0],
            &mut graph,
        );
        let image = egui_overlay(&self.gui, background, &mut graph);
        graph.export(image);
        graph
    }

    fn on_window_event(&mut self, event: &WindowEvent, context: &mut WindowContext<'_>) {
        self.gui.on_window_event(event, context);
    }
}
