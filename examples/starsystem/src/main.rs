use gilgamesh::blocks::{bloom, rasterize, tone_map};
use gilgamesh::camera::{Camera, Perspective};
use gilgamesh::core::app::{App, FrameContext};
use gilgamesh::core::render_context::FrameBindingsData;
use gilgamesh::core::render_graph::{RenderGraph, TextureFormat};
use gilgamesh::core::wgpu_context::WGPUContext;
use gilgamesh::core::window_runtime::WindowRuntime;
use gilgamesh::geometry::mesh::Mesh;
use gilgamesh::geometry::primitive;
use gilgamesh::input::orbit_control::OrbitControl;
use gilgamesh::lights::debug::create_point_light_debug_mesh;
use gilgamesh::lights::directional_light::DirectionalLight;
use gilgamesh::lights::light::Light;
use gilgamesh::lights::point_light::PointLight;
use gilgamesh::materials::pbr::PbrMaterial;
use gilgamesh::texture::{Linear, Srgb, Texture};
use gilgamesh::transform::Transformable;
use glam::{DQuat, DVec3};
use std::rc::Rc;

fn main() {
    WindowRuntime::<StarSystemDemo>::run(1000, 800);
}

struct StarSystemDemo {
    camera: Camera<Perspective>,
    orbit_control: OrbitControl,
    sun: Mesh,
    earth: Mesh,
    plane: Mesh,
    directional_light: DirectionalLight,
    point_lights: [PointLight; 4],
    point_light_meshes: [Mesh; 4],
}

impl App for StarSystemDemo {
    fn new(ctx: &WGPUContext) -> Self {
        let camera = Camera::default();
        let mut orbit_control = OrbitControl::default();
        orbit_control.set_radius(2.0);

        let mut directional_light = DirectionalLight::default();
        directional_light.set_intensity(0.0);

        let point_lights = [
            (1.0, 0.0, 0.0),
            (0.0, 0.0, 1.0),
            (0.0, 1.0, 0.0),
            (1.0, 1.0, 0.0),
        ]
        .map(|(r, g, b)| {
            let mut light = PointLight::default();
            light.set_color(r, g, b);
            light.set_intensity(200.0);
            light
        });
        let point_light_meshes = point_lights
            .each_ref()
            .map(|light| create_point_light_debug_mesh(light, ctx));

        let sun_texture = Rc::new(Texture::<Srgb>::new(
            "Sun texture",
            include_bytes!("../assets/sun.jpg"),
            ctx,
        ));
        let mut sun_material = PbrMaterial::new("SunMaterial");
        sun_material.set_albedo_texture(sun_texture);
        let sun = Mesh::from_cpu_buffers(
            "Sun",
            primitive::sphere(32),
            Rc::new(sun_material.into()),
            ctx,
        );

        let mut earth_material = PbrMaterial::new("EarthMaterial");
        earth_material.metallic = 0.0;
        let earth_diffuse_texture = Rc::new(Texture::<Srgb>::new(
            "Earth diffuse texture",
            include_bytes!("../assets/2k_earth_daymap.jpg"),
            ctx,
        ));
        earth_material.set_albedo_texture(earth_diffuse_texture);
        earth_material.set_normal_map(Rc::new(Texture::<Linear>::new(
            "Earth normal map",
            include_bytes!("../assets/2k_earth_normal_map.png"),
            ctx,
        )));
        earth_material.set_roughness_map(Rc::new(Texture::<Linear>::new(
            "Earth roughness map",
            include_bytes!("../assets/2k_earth_roughness_map.png"),
            ctx,
        )));
        let earth = Mesh::from_cpu_buffers(
            "Earth",
            primitive::sphere(32),
            Rc::new(earth_material.into()),
            ctx,
        );
        earth.transform_mut().parent = Some(sun.transform_rc());

        camera.transform_mut().parent = Some(earth.transform_rc());

        let mut plane_material = PbrMaterial::new("PlaneMaterial");
        plane_material.metallic = 0.0;
        plane_material.roughness = 0.8;
        let plane = Mesh::from_cpu_buffers(
            "Plane",
            primitive::plane(10, 10.0),
            Rc::new(plane_material.into()),
            ctx,
        );
        plane.transform_mut().position = DVec3::new(0.0, -5.0, 0.0);

        Self {
            camera,
            orbit_control,
            sun,
            earth,
            plane,
            directional_light,
            point_lights,
            point_light_meshes,
        }
    }

    fn name(&self) -> &str {
        "Gilgamesh"
    }

    fn update_before_render(&mut self, frame: &FrameContext<'_>) {
        self.camera.projection.aspect_ratio =
            frame.render_width as f64 / frame.render_height as f64;
        self.orbit_control
            .update(&mut self.camera, frame.mouse, frame.delta_seconds);
        self.sun.transform_mut().rotation = DQuat::from_rotation_y(frame.elapsed_seconds);
        self.earth.transform_mut().position = DVec3::new(6.0, 0.0, 0.0);

        let orbit_angle = 2.0 * frame.elapsed_seconds;
        let (orbit_sin, orbit_cos) = orbit_angle.sin_cos();
        let [red_light, blue_light, green_light, yellow_light] = &self.point_lights;
        red_light.transform_mut().position = DVec3::new(12.0 * orbit_cos, 0.0, 12.0 * orbit_sin);
        blue_light.transform_mut().position = DVec3::new(12.0 * orbit_sin, 12.0 * orbit_cos, 0.0);
        green_light.transform_mut().position = DVec3::new(0.0, 12.0 * orbit_cos, -12.0 * orbit_sin);
        yellow_light.transform_mut().position = DVec3::new(15.0 * orbit_sin, 0.0, 15.0 * orbit_cos);
    }

    fn render_graph(&self) -> RenderGraph<'_> {
        let mut graph = RenderGraph::new();
        let scene = rasterize(
            FrameBindingsData::new(&self.camera, &self.directional_light, &self.point_lights),
            self.point_light_meshes
                .iter()
                .chain([&self.sun, &self.earth, &self.plane]),
            TextureFormat::Rgba16Float,
            &mut graph,
        );
        let color = bloom(scene.color, 5, 0.04, &mut graph);
        let image = tone_map(color, TextureFormat::Rgba8UnormSrgb, &mut graph);
        graph.export(image);
        graph
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::StarSystemDemo;
    use gilgamesh::core::headless_runtime::HeadlessRuntime;
    use gilgamesh::lights::light::Light;
    use std::time::Duration;

    #[test]
    #[ignore = "GPU end-to-end check; run explicitly with approval"]
    fn earth_maps_render_in_the_star_system_graph() {
        HeadlessRuntime::<StarSystemDemo>::run(400, 320, async |mut runtime| {
            for light in &mut runtime.app_mut().point_lights {
                light.set_intensity(200.0);
            }
            runtime.render(Duration::ZERO);
            let image = runtime.capture_frame().await.unwrap();
            assert_eq!(image.dimensions(), (400, 320));
            assert!(image.pixels().any(|p| p[0] > 20 && p[2] > 20));
            runtime.resize(321, 257);
            runtime.render(Duration::from_millis(16));
            let image = runtime.capture_frame().await.unwrap();
            assert_eq!(image.dimensions(), (321, 257));
            assert!(image.pixels().any(|p| p[0] > 20 && p[2] > 20));
        });
    }
}
