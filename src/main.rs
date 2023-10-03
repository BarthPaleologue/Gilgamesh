extern crate gilgamesh;

use winit::event::VirtualKeyCode::*;
use gilgamesh::camera::camera::Camera;
use gilgamesh::input::transform_control::OrbitControl;
use gilgamesh::core::engine::Engine;
use gilgamesh::core::scene::Scene;
use gilgamesh::geometry::primitive::PrimitiveMesh;
use gilgamesh::lights::debug::show_point_light_debug_mesh;
use gilgamesh::lights::light::Light;
use gilgamesh::lights::point_light::PointLight;
use gilgamesh::transform::{Transformable};

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn run() {
    let (mut engine, event_loop) = Engine::new("Gilgamesh", 1000, 800);

    let mut scene = Scene::new(&engine);

    let mut camera = Camera::new(&engine);
    camera.control = Some(Box::<OrbitControl>::default());

    scene.directional_light.set_intensity(0.0);

    let light_intensity = 0.2;

    let mut point_light = PointLight::default();
    point_light.set_color(1.0, 0.0, 0.0);
    point_light.set_intensity(light_intensity);
    show_point_light_debug_mesh(&point_light, &mut scene, &mut engine);
    let point_light1_idx = scene.add_point_light(point_light);

    let mut point_light2 = PointLight::default();
    point_light2.set_color(0.0, 0.0, 1.0);
    point_light2.set_intensity(light_intensity);
    show_point_light_debug_mesh(&point_light2, &mut scene, &mut engine);
    let point_light2_idx = scene.add_point_light(point_light2);

    let mut point_light3 = PointLight::default();
    point_light3.set_color(0.0, 1.0, 0.0);
    point_light3.set_intensity(light_intensity);
    show_point_light_debug_mesh(&point_light3, &mut scene, &mut engine);
    let point_light3_idx = scene.add_point_light(point_light3);

    let mut point_light4 = PointLight::default();
    point_light4.set_color(1.0, 1.0, 0.0);
    point_light4.set_intensity(light_intensity);
    show_point_light_debug_mesh(&point_light4, &mut scene, &mut engine);
    let point_light4_idx = scene.add_point_light(point_light4);

    let mut sun = PrimitiveMesh::sphere("Sun", 32, &mut engine);
    sun.material.set_ambient_texture("textures/sun.jpg", &mut engine.wgpu_context);
    let sun_idx = scene.add_mesh(sun);

    let mut earth = PrimitiveMesh::sphere("Earth", 32, &mut engine);
    earth.transform_mut().parent = Some(scene.meshes[sun_idx].transform_rc());
    earth.material.set_diffuse_texture("textures/2k_earth_daymap.jpg", &mut engine.wgpu_context);
    earth.material.set_specular_texture("textures/2k_earth_specular_map.jpg", &mut engine.wgpu_context);
    earth.material.set_normal_map("textures/2k_earth_normal_map.jpg", &mut engine.wgpu_context);
    //earth.material.set_polygon_mode(wgpu::PolygonMode::Line);
    camera.transform_mut().parent = Some(earth.transform_rc());
    let earth_idx = scene.add_mesh(earth);

    let plane = PrimitiveMesh::plane("Plane", 10, 10.0, &mut engine);
    plane.transform_mut().set_position(0.0, -5.0, 0.0);
    let plane_idx = scene.add_mesh(plane);


    scene.set_active_camera(camera);

    scene.on_before_render.push(Box::new(move |engine, active_camera, meshes, point_lights, mouse| {
        meshes[sun_idx].transform_mut().set_rotation(
            0.0,
            engine.get_elapsed_time(),
            0.0,
        );

        meshes[earth_idx].transform_mut().set_position(
            6.0,
            0.0,
            0.0,
        );

        /*meshes[earth_idx].transform_mut().set_rotation(
            0.0,
            2.0 * engine.get_elapsed_time(),
            0.0,
        );*/

        point_lights[point_light1_idx].transform_mut().set_position(
            12.0 * f32::cos(2.0 * engine.get_elapsed_time()),
            0.0,
            12.0 * f32::sin(2.0 * engine.get_elapsed_time()),
        );

        point_lights[point_light2_idx].transform_mut().set_position(
            12.0 * f32::sin(2.0 * engine.get_elapsed_time()),
            12.0 * f32::cos(2.0 * engine.get_elapsed_time()),
            0.0,
        );

        point_lights[point_light3_idx].transform_mut().set_position(
            0.0,
            12.0 * f32::cos(-2.0 * engine.get_elapsed_time()),
            12.0 * f32::sin(-2.0 * engine.get_elapsed_time()),
        );

        point_lights[point_light4_idx].transform_mut().set_position(
            15.0 * f32::sin(2.0 * engine.get_elapsed_time()),
            0.0,
            15.0 * f32::cos(2.0 * engine.get_elapsed_time()),
        );
    }));

    //scene.on_key_pressed.push(Box::new(|engine, active_camera, key| {}));

    engine.start(scene, event_loop);
}

fn main() {
    run();
}