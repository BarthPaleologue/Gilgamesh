use gilgamesh::blocks::{expose, rasterize, tone_map, vertical_split_screen, visualize_depth};
use gilgamesh::camera::{Camera, Perspective};
use gilgamesh::core::render_graph::{RenderGraph, TextureFormat};
use gilgamesh::core::{
    App, FrameBindingsData, FrameContext, WGPUContext, WindowContext, WindowRuntime,
};
use gilgamesh::geometry::{Mesh, primitive};
use gilgamesh::input::OrbitControl;
use gilgamesh::lights::DirectionalLight;
use gilgamesh::materials::BlinnPhongMaterial;
use gilgamesh::photometry::Nits;
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
    directional_lights: [DirectionalLight; 1],
    split: f32,
}

impl App for SplitScreenDemo {
    fn new(ctx: &WGPUContext) -> Self {
        let mut ground_material = BlinnPhongMaterial::new("Ground");
        ground_material.set_diffuse_color(0.3, 0.35, 0.4);
        ground_material.set_ambient_luminance(Nits::new(10_000.0));
        ground_material.set_ambient_color(0.02, 0.02, 0.02);
        let ground = Mesh::from_cpu_buffers(
            "Ground",
            &primitive::plane(2, 14.0),
            Rc::new(ground_material.into()),
            ctx,
        );

        let mut sphere_material = BlinnPhongMaterial::new("Amber");
        sphere_material.set_diffuse_color(0.9, 0.2, 0.05);
        sphere_material.set_ambient_luminance(Nits::new(10_000.0));
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
        cube_material.set_ambient_luminance(Nits::new(10_000.0));
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
            directional_lights: [DirectionalLight::default()],
            split: 0.5,
        }
    }

    fn name(&self) -> &'static str {
        "Color | Raw reverse-Z depth — move mouse to split, drag to orbit, scroll to zoom"
    }

    fn update_before_frame(&mut self, frame: &FrameContext<'_>) {
        self.orbit_control
            .update(&mut self.camera, frame.mouse, frame.delta_seconds);
    }

    fn frame_graph(&self) -> RenderGraph<'_> {
        let mut graph = RenderGraph::new();
        let scene = rasterize(
            FrameBindingsData::new(&self.camera, &self.directional_lights, &[]),
            &self.meshes,
            TextureFormat::Rgba32Float,
            &mut graph,
        );
        let exposed = expose(
            scene.color,
            self.camera.exposure,
            TextureFormat::Rgba16Float,
            &mut graph,
        );
        let color = tone_map(exposed, TextureFormat::Rgba8UnormSrgb, &mut graph);
        let depth = visualize_depth(scene.depth, &mut graph);
        let image = vertical_split_screen(color, depth, self.split, &mut graph);
        graph.export(image);
        graph
    }

    fn on_window_event(&mut self, event: &WindowEvent, context: &mut WindowContext<'_>) {
        if let WindowEvent::CursorMoved { position, .. } = event {
            let width = f64::from(context.frame.output().texture().width());
            #[expect(
                clippy::cast_possible_truncation,
                reason = "A split fraction in [0, 1] only needs shader f32 precision"
            )]
            let split = (position.x / width).clamp(0.0, 1.0) as f32;
            self.split = split;
        }
    }
}

#[cfg(test)]
mod tests;
