use crate::camera::Camera;
use crate::core::app::{App, FrameContext};
use crate::core::engine::Engine;
use crate::core::render_context::RenderContext;
use crate::core::wgpu_context::WGPUContext;
use crate::geometry::mesh::Mesh;
use crate::geometry::primitive::PrimitiveMesh;
use crate::input::transform_control::OrbitControl;
use crate::lights::debug::create_point_light_debug_mesh;
use crate::lights::directional_light::DirectionalLight;
use crate::lights::light::Light;
use crate::lights::point_light::PointLight;
use crate::materials::blinn_phong::BlinnPhongMaterial;
use crate::materials::pbr::PbrMaterial;
use crate::texture::Texture;
use crate::transform::Transformable;
use std::rc::Rc;

pub fn run() {
    Engine::run("Gilgamesh", 1000, 800, StarSystemDemo::new);
}

pub struct StarSystemDemo {
    camera: Camera,
    sun: Mesh,
    earth: Mesh,
    plane: Mesh,
    directional_light: DirectionalLight,
    point_lights: [PointLight; 4],
    point_light_meshes: [Mesh; 4],
}

impl StarSystemDemo {
    pub fn new(ctx: &WGPUContext) -> Self {
        let mut camera = Camera::new(ctx.config.width as f32 / ctx.config.height as f32);
        let mut camera_control = OrbitControl::default();
        camera_control.set_radius(2.0);
        camera.control = Some(Box::new(camera_control));

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

        let mut sun = PrimitiveMesh::sphere("Sun", 32, ctx);
        let sun_texture = Rc::new(Texture::new("Sun texture", "textures/sun.jpg", ctx));
        let mut sun_material = PbrMaterial::new("SunMaterial", ctx);
        sun_material.set_albedo_texture(sun_texture);
        sun.set_material(Box::new(sun_material));

        let mut earth = PrimitiveMesh::sphere("Earth", 32, ctx);
        earth.transform_mut().parent = Some(sun.transform_rc());
        let mut earth_material = BlinnPhongMaterial::new("EarthMaterial", ctx);
        let earth_diffuse_texture = Rc::new(Texture::new(
            "Earth diffuse texture",
            "textures/2k_earth_daymap.jpg",
            ctx,
        ));
        earth_material.set_diffuse_texture(earth_diffuse_texture);
        let earth_specular_texture = Rc::new(Texture::new(
            "Earth specular texture",
            "textures/2k_earth_specular_map.jpg",
            ctx,
        ));
        earth_material.set_specular_texture(earth_specular_texture);
        earth.set_material(Box::new(earth_material));

        camera.transform_mut().parent = Some(earth.transform_rc());

        let plane = PrimitiveMesh::plane("Plane", 10, 10.0, ctx);
        plane.transform_mut().set_position(0.0, -5.0, 0.0);

        Self {
            camera,
            sun,
            earth,
            plane,
            directional_light,
            point_lights,
            point_light_meshes,
        }
    }
}

impl App for StarSystemDemo {
    fn update_before_render(&mut self, frame: &FrameContext<'_>) {
        self.camera
            .listen_to_control(frame.mouse, frame.delta_seconds);
        self.sun
            .transform_mut()
            .set_rotation(0.0, frame.elapsed_seconds, 0.0);
        self.earth.transform_mut().set_position(6.0, 0.0, 0.0);

        let [red_light, blue_light, green_light, yellow_light] = &self.point_lights;
        red_light.transform_mut().set_position(
            12.0 * f32::cos(2.0 * frame.elapsed_seconds),
            0.0,
            12.0 * f32::sin(2.0 * frame.elapsed_seconds),
        );
        blue_light.transform_mut().set_position(
            12.0 * f32::sin(2.0 * frame.elapsed_seconds),
            12.0 * f32::cos(2.0 * frame.elapsed_seconds),
            0.0,
        );
        green_light.transform_mut().set_position(
            0.0,
            12.0 * f32::cos(-2.0 * frame.elapsed_seconds),
            12.0 * f32::sin(-2.0 * frame.elapsed_seconds),
        );
        yellow_light.transform_mut().set_position(
            15.0 * f32::sin(2.0 * frame.elapsed_seconds),
            0.0,
            15.0 * f32::cos(2.0 * frame.elapsed_seconds),
        );
    }

    fn get_render_context(&mut self) -> RenderContext<'_, impl IntoIterator<Item = &mut Mesh>> {
        RenderContext::new(
            &mut self.camera,
            self.point_light_meshes.iter_mut().chain([
                &mut self.sun,
                &mut self.earth,
                &mut self.plane,
            ]),
            &self.directional_light,
            &self.point_lights,
        )
    }
}
