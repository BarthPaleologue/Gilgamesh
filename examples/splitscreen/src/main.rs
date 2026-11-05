use gilgamesh::blocks::{rasterize, tone_map, vertical_split_screen, visualize_depth};
use gilgamesh::camera::{Camera, Perspective};
use gilgamesh::core::app::{App, FrameContext};
use gilgamesh::core::render_context::FrameBindingsData;
use gilgamesh::core::render_graph::{RenderGraph, TextureFormat};
use gilgamesh::core::wgpu_context::WGPUContext;
use gilgamesh::core::window_runtime::{WindowContext, WindowRuntime};
use gilgamesh::geometry::{mesh::Mesh, primitive};
use gilgamesh::input::orbit_control::OrbitControl;
use gilgamesh::lights::directional_light::DirectionalLight;
use gilgamesh::materials::blinn_phong::BlinnPhongMaterial;
use gilgamesh::transform::Transformable;
use glam::DVec3;
use std::rc::Rc;
use winit::event::WindowEvent;

fn main() {
    WindowRuntime::<SplitScreenDemo>::run(1000, 700);
}

struct SplitScreenDemo {
    camera: Camera<Perspective>,
    orbit_control: OrbitControl,
    meshes: [Mesh; 3],
    light: DirectionalLight,
    split: f32,
}

impl App for SplitScreenDemo {
    fn new(ctx: &WGPUContext) -> Self {
        let mut ground_material = BlinnPhongMaterial::new("Ground");
        ground_material.set_diffuse_color(0.3, 0.35, 0.4);
        ground_material.set_ambient_color(0.02, 0.02, 0.02);
        let ground = Mesh::from_cpu_buffers(
            "Ground",
            &primitive::plane(2, 14.0),
            Rc::new(ground_material.into()),
            ctx,
        );

        let mut sphere_material = BlinnPhongMaterial::new("Amber");
        sphere_material.set_diffuse_color(0.9, 0.2, 0.05);
        sphere_material.set_ambient_color(0.03, 0.01, 0.005);
        let sphere = Mesh::from_cpu_buffers(
            "Sphere",
            &primitive::sphere(32),
            Rc::new(sphere_material.into()),
            ctx,
        );
        sphere.transform_mut().position = DVec3::Y;

        let mut cube_material = BlinnPhongMaterial::new("Blue");
        cube_material.set_diffuse_color(0.1, 0.45, 0.9);
        cube_material.set_ambient_color(0.01, 0.02, 0.03);
        let cube = Mesh::from_cpu_buffers(
            "Cube",
            &primitive::cube(),
            Rc::new(cube_material.into()),
            ctx,
        );
        cube.transform_mut().position = DVec3::new(1.0, 0.5, 1.4);

        Self {
            camera: Camera::default(),
            orbit_control: OrbitControl::default(),
            meshes: [ground, sphere, cube],
            light: DirectionalLight::default(),
            split: 0.5,
        }
    }

    fn name(&self) -> &'static str {
        "Color | Raw reverse-Z depth — move mouse to split, drag to orbit, scroll to zoom"
    }

    fn update_before_render(&mut self, frame: &FrameContext<'_>) {
        self.camera.projection.aspect_ratio =
            f64::from(frame.render_width) / f64::from(frame.render_height);
        self.orbit_control
            .update(&mut self.camera, frame.mouse, frame.delta_seconds);
    }

    fn render_graph(&self) -> RenderGraph<'_> {
        let mut graph = RenderGraph::new();
        let scene = rasterize(
            FrameBindingsData::new(&self.camera, &self.light, &[]),
            &self.meshes,
            TextureFormat::Rgba16Float,
            &mut graph,
        );
        let color = tone_map(scene.color, TextureFormat::Rgba8UnormSrgb, &mut graph);
        let depth = visualize_depth(scene.depth, &mut graph);
        let image = vertical_split_screen(color, depth, self.split, &mut graph);
        graph.export(image);
        graph
    }

    fn on_window_event(&mut self, event: &WindowEvent, context: &mut WindowContext<'_>) {
        if let WindowEvent::CursorMoved { position, .. } = event {
            let width = f64::from(context.frame.output().texture().width());
            self.split = (position.x / width).clamp(0.0, 1.0) as f32;
        }
    }
}

#[cfg(test)]
mod tests;
