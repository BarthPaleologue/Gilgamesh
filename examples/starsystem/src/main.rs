use gilgamesh::blocks::{bloom, rasterize, tone_map};
use gilgamesh::camera::{Camera, Perspective};
use gilgamesh::core::app::{App, FrameContext};
use gilgamesh::core::render_context::FrameBindingsData;
use gilgamesh::core::render_graph::{RenderGraph, TextureFormat};
use gilgamesh::core::wgpu_context::WGPUContext;
use gilgamesh::core::window_runtime::WindowRuntime;
use gilgamesh::embed_texture;
use gilgamesh::geometry::mesh::Mesh;
use gilgamesh::geometry::primitive;
use gilgamesh::input::orbit_control::OrbitControl;
use gilgamesh::lights::debug::create_point_light_debug_mesh;
use gilgamesh::lights::directional_light::DirectionalLight;
use gilgamesh::lights::light::Light;
use gilgamesh::lights::point_light::PointLight;
use gilgamesh::materials::pbr::PbrMaterial;
use gilgamesh::texture::{Linear, Srgb};
use gilgamesh::transform::{Transform, Transformable};
use glam::{DQuat, DVec3};
use std::cell::RefCell;
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
    point_lights: [PointLight; 5],
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

        let sun_texture = Rc::new(embed_texture!("Sun", "../assets/sun.jpg", Srgb, ctx));
        let mut sun_material = PbrMaterial::new("SunMaterial");
        sun_material.set_albedo_color(0.0, 0.0, 0.0);
        sun_material.metallic = 0.0;
        sun_material.set_emissive_texture(sun_texture);
        sun_material.set_emissive_color(20.0, 20.0, 20.0);
        let sun = Mesh::from_cpu_buffers(
            "Sun",
            &primitive::sphere(32),
            Rc::new(sun_material.into()),
            ctx,
        );

        let mut sun_light = PointLight::default();
        sun_light.set_intensity(200.0);
        sun_light.transform_mut().parent = Some(sun.transform_rc());
        let [red_light, blue_light, green_light, yellow_light] = point_lights;
        let point_lights = [red_light, blue_light, green_light, yellow_light, sun_light];

        let mut earth_material = PbrMaterial::new("EarthMaterial");
        earth_material.metallic = 0.0;
        let earth_diffuse_texture = Rc::new(embed_texture!(
            "Earth diffuse",
            "../assets/2k_earth_daymap.jpg",
            Srgb,
            ctx,
        ));
        earth_material.set_albedo_texture(earth_diffuse_texture);
        earth_material.set_normal_map(Rc::new(embed_texture!(
            "Earth normal",
            "../assets/2k_earth_normal_map.png",
            Linear,
            ctx,
        )));
        earth_material.set_roughness_map(Rc::new(embed_texture!(
            "Earth roughness",
            "../assets/2k_earth_roughness_map.png",
            Linear,
            ctx,
        )));
        let earth = Mesh::from_cpu_buffers(
            "Earth",
            &primitive::sphere(32),
            Rc::new(earth_material.into()),
            ctx,
        );
        let earth_orbit = Rc::new(RefCell::new(Transform {
            position: DVec3::new(6.0, 0.0, 0.0),
            parent: Some(sun.transform_rc()),
            ..Default::default()
        }));
        earth.transform_mut().parent = Some(earth_orbit.clone());
        camera.transform_mut().parent = Some(earth_orbit);

        let mut plane_material = PbrMaterial::new("PlaneMaterial");
        plane_material.metallic = 0.0;
        plane_material.roughness = 0.8;
        let plane = Mesh::from_cpu_buffers(
            "Plane",
            &primitive::plane(10, 40.0),
            Rc::new(plane_material.into()),
            ctx,
        );
        plane.transform_mut().position = DVec3::new(0.0, -15.0, 0.0);

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

    fn name(&self) -> &'static str {
        "Gilgamesh"
    }

    fn update_before_render(&mut self, frame: &FrameContext<'_>) {
        self.camera.projection.aspect_ratio =
            f64::from(frame.render_width) / f64::from(frame.render_height);
        self.orbit_control
            .update(&mut self.camera, frame.mouse, frame.delta_seconds);
        self.sun.transform_mut().rotation = DQuat::from_rotation_y(0.2 * frame.elapsed_seconds);
        self.earth.transform_mut().rotation = DQuat::from_rotation_y(0.5 * frame.elapsed_seconds);

        let orbit_angle = 0.5 * frame.elapsed_seconds;
        let (orbit_sin, orbit_cos) = orbit_angle.sin_cos();
        let [red_light, blue_light, green_light, yellow_light, _] = &self.point_lights;
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
    fn earth_spins_under_the_orbit_camera_and_captures_keep_its_pose() {
        use gilgamesh::transform::Transformable;
        use std::rc::Rc;

        HeadlessRuntime::<StarSystemDemo>::run(400, 320, async |mut runtime| {
            for light in &mut runtime.app_mut().point_lights[..4] {
                light.set_intensity(0.0);
            }
            runtime.render(Duration::ZERO);
            let app = runtime.app();
            assert!(Rc::ptr_eq(
                app.camera.transform().parent.as_ref().unwrap(),
                app.earth.transform().parent.as_ref().unwrap(),
            ));
            let initial_rotation = app.earth.transform().rotation;
            let camera_rotation = app.camera.transform().rotation;
            let earth_in_view = app
                .camera
                .transform()
                .world_matrix()
                .inverse()
                .transform_point3(app.earth.transform().world_position());
            let before = runtime.capture_frame().await.unwrap();

            runtime.render(Duration::from_secs(2));
            let app = runtime.app();
            assert!(
                !app.earth
                    .transform()
                    .rotation
                    .abs_diff_eq(initial_rotation, 1e-10)
            );
            assert!(
                app.camera
                    .transform()
                    .rotation
                    .abs_diff_eq(camera_rotation, 1e-10)
            );
            let next_earth_in_view = app
                .camera
                .transform()
                .world_matrix()
                .inverse()
                .transform_point3(app.earth.transform().world_position());
            assert!(next_earth_in_view.abs_diff_eq(earth_in_view, 1e-10));
            let after = runtime.capture_frame().await.unwrap();
            assert!(
                after.enumerate_pixels().any(|(x, y, pixel)| {
                    (130..270).contains(&x)
                        && (100..220).contains(&y)
                        && (0..3).any(|c| pixel[c].abs_diff(before.get_pixel(x, y)[c]) > 5)
                }),
                "Earth's surface must visibly rotate independently of the camera"
            );

            let rotation = runtime.app().earth.transform().rotation;
            let capture = runtime.render_image(400, 320).await.unwrap();
            assert_eq!(runtime.app().earth.transform().rotation, rotation);
            assert_eq!(capture, after);
        });
    }

    #[test]
    #[ignore = "GPU end-to-end check; run explicitly with approval"]
    fn solar_light_follows_the_sun_and_illuminates_earth() {
        use gilgamesh::transform::Transformable;
        use glam::DVec3;
        use std::rc::Rc;

        HeadlessRuntime::<StarSystemDemo>::run(400, 320, async |mut runtime| {
            let app = runtime.app_mut();
            for light in &mut app.point_lights[..4] {
                light.set_intensity(0.0);
            }
            let solar_light = &app.point_lights[4];
            assert!(Rc::ptr_eq(
                solar_light.transform().parent.as_ref().unwrap(),
                &app.sun.transform_rc()
            ));
            app.sun.transform_mut().position = DVec3::new(3.0, 0.0, 2.0);
            assert_eq!(
                solar_light.transform().world_position(),
                app.sun.transform().world_position()
            );
            runtime.render(Duration::ZERO);
            let lit = runtime.capture_frame().await.unwrap();
            runtime.app_mut().point_lights[4].set_intensity(0.0);
            runtime.render(Duration::ZERO);
            let unlit = runtime.capture_frame().await.unwrap();
            assert!(
                lit.enumerate_pixels().any(|(x, y, pixel)| {
                    (130..270).contains(&x)
                        && (100..220).contains(&y)
                        && pixel[0].saturating_sub(unlit.get_pixel(x, y)[0]) > 10
                }),
                "The solar point light must illuminate the Earth independently of the emissive surface"
            );
        });
    }

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
