use gilgamesh::camera::{Camera, CameraProjection, Perspective};
use gilgamesh::core::app::{App, FrameContext};
use gilgamesh::core::engine::Engine;
use gilgamesh::core::render_context::FrameBindingsData;
use gilgamesh::core::wgpu_context::WGPUContext;
use gilgamesh::geometry::mesh::Mesh;
use gilgamesh::geometry::primitive;
use gilgamesh::input::orbit_control::OrbitControl;
use gilgamesh::lights::debug::create_point_light_debug_mesh;
use gilgamesh::lights::directional_light::DirectionalLight;
use gilgamesh::lights::light::Light;
use gilgamesh::lights::point_light::PointLight;
use gilgamesh::materials::blinn_phong::BlinnPhongMaterial;
use gilgamesh::materials::pbr::PbrMaterial;
use gilgamesh::texture::Texture;
use gilgamesh::transform::Transformable;
use glam::{DQuat, DVec3};
use std::rc::Rc;

fn main() {
    Engine::<StarSystemDemo>::run("Gilgamesh", 1000, 800);
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
            light.set_intensity(0.2);
            light
        });
        let point_light_meshes = point_lights
            .each_ref()
            .map(|light| create_point_light_debug_mesh(light, ctx));

        let sun_texture = Rc::new(Texture::new(
            "Sun texture",
            include_bytes!("assets/sun.jpg"),
            ctx,
        ));
        let mut sun_material = PbrMaterial::new("SunMaterial", ctx);
        sun_material.set_albedo_texture(sun_texture);
        let sun = Mesh::from_cpu_buffers("Sun", primitive::sphere(32), Rc::new(sun_material), ctx);

        let mut earth_material = BlinnPhongMaterial::new("EarthMaterial", ctx);
        let earth_diffuse_texture = Rc::new(Texture::new(
            "Earth diffuse texture",
            include_bytes!("assets/2k_earth_daymap.jpg"),
            ctx,
        ));
        earth_material.set_diffuse_texture(earth_diffuse_texture);
        let earth_specular_texture = Rc::new(Texture::new(
            "Earth specular texture",
            include_bytes!("assets/2k_earth_specular_map.jpg"),
            ctx,
        ));
        earth_material.set_specular_texture(earth_specular_texture);
        let earth =
            Mesh::from_cpu_buffers("Earth", primitive::sphere(32), Rc::new(earth_material), ctx);
        earth.transform_mut().parent = Some(sun.transform_rc());

        camera.transform_mut().parent = Some(earth.transform_rc());

        let plane = Mesh::from_cpu_buffers(
            "Plane",
            primitive::plane(10, 10.0),
            Rc::new(BlinnPhongMaterial::new("PlaneMaterial", ctx)),
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

    fn update_before_render(&mut self, frame: &FrameContext<'_>) {
        self.camera.projection.aspect_ratio =
            frame.surface_width as f64 / frame.surface_height as f64;
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

    fn get_frame_bindings_data(&mut self) -> FrameBindingsData<'_, impl CameraProjection> {
        FrameBindingsData::new(&self.camera, &self.directional_light, &self.point_lights)
    }

    fn get_meshes(&mut self) -> impl IntoIterator<Item = &mut Mesh> {
        self.point_light_meshes
            .iter_mut()
            .chain([&mut self.sun, &mut self.earth, &mut self.plane])
    }
}
