use gilgamesh::camera::Camera;
use gilgamesh::camera::uniforms::CameraUniforms;
use gilgamesh::geometry::mesh::{Vertex, zip_vertex_data};
use gilgamesh::geometry::vertex_data::VertexData;
use gilgamesh::input::mouse::Mouse;
use gilgamesh::input::transform_control::OrbitControl;
use gilgamesh::lights::directional_light::{DirectionalLight, DirectionalLightUniform};
use gilgamesh::lights::light::Light;
use gilgamesh::lights::point_light::{PointLight, PointLightUniforms};
use gilgamesh::transform::{Transform, TransformUniforms, Transformable};
use glam::{DQuat, DVec3, DVec4};
use std::cell::RefCell;
use std::f64::consts::FRAC_PI_2;
use std::rc::Rc;

fn assert_close<const N: usize>(actual: [f64; N], expected: [f64; N]) {
    for (actual, expected) in actual.into_iter().zip(expected) {
        assert!(
            (actual - expected).abs() < 1e-12,
            "expected {expected}, got {actual}"
        );
    }
}

#[test]
fn default_transform_has_identity_orientation() {
    let transform = Transform::new();
    assert_eq!(transform.rotation, DQuat::IDENTITY);
    let point = DVec4::new(1.0, -2.0, 3.0, 1.0);
    assert_close(
        (transform.compute_world_matrix() * point).into(),
        point.into(),
    );
}

#[test]
fn world_transform_keeps_rotation_then_nonuniform_scale_then_translation() {
    let mut transform = Transform::new();
    transform.position = DVec3::new(3.0, -2.0, 4.0);
    transform.scaling = DVec3::new(2.0, 3.0, 4.0);
    transform.rotation = DQuat::from_rotation_z(FRAC_PI_2) * DQuat::from_rotation_x(FRAC_PI_2);
    let position = transform.compute_world_matrix() * DVec4::new(1.0, 2.0, 3.0, 1.0);
    assert_close(position.into(), [9.0, 1.0, 12.0, 1.0]);
}

#[test]
fn child_transform_follows_changes_to_its_parent() {
    let parent = Rc::new(RefCell::new(Transform::new()));
    parent.borrow_mut().position = DVec3::new(5.0, -4.0, 2.0);
    parent.borrow_mut().scaling = DVec3::new(2.0, 3.0, 4.0);
    parent.borrow_mut().rotation = DQuat::from_rotation_z(FRAC_PI_2);
    let mut child = Transform::new();
    child.parent = Some(parent.clone());
    child.position = DVec3::new(1.0, 2.0, 3.0);
    let position = child.compute_world_matrix() * DVec4::new(0.0, 0.0, 0.0, 1.0);
    assert_close(position.into(), [1.0, -1.0, 14.0, 1.0]);

    parent.borrow_mut().position = DVec3::new(8.0, -4.0, 2.0);
    let position = child.compute_world_matrix() * DVec4::new(0.0, 0.0, 0.0, 1.0);
    assert_close(position.into(), [4.0, -1.0, 14.0, 1.0]);
}

#[test]
fn normal_transform_accounts_for_nonuniform_scale_and_rotation() {
    let mut transform = Transform::new();
    transform.scaling = DVec3::new(2.0, 4.0, 8.0);
    transform.rotation = DQuat::from_rotation_z(FRAC_PI_2);
    let normal = transform.compute_normal_matrix() * DVec4::new(1.0, 0.0, 0.0, 0.0);
    assert_close(normal.into(), [0.0, 0.25, 0.0, 0.0]);
}

#[test]
fn local_axes_only_depend_on_rotation_and_stay_normalized() {
    let mut transform = Transform::new();
    transform.position = DVec3::new(100.0, -20.0, 50.0);
    transform.scaling = DVec3::new(2.0, 3.0, 4.0);
    transform.rotation = DQuat::from_rotation_y(FRAC_PI_2);
    assert_close(transform.forward().into(), [1.0, 0.0, 0.0]);
    assert_close(transform.right().into(), [0.0, 0.0, -1.0]);
    assert_close(transform.up().into(), [0.0, 1.0, 0.0]);
    assert_close(
        transform
            .local_direction_to_world(DVec3::new(0.0, 0.0, 5.0))
            .into(),
        [1.0, 0.0, 0.0],
    );
}

#[test]
fn projection_uses_degrees_and_webgpu_depth_range() {
    let mut camera = Camera::new(1.5);
    camera.fov = 60.0;
    camera.z_near = 0.1;
    camera.z_far = 10.0;
    let projection = camera.projection_matrix();
    for (distance, expected_depth) in [(camera.z_near, 0.0), (camera.z_far, 1.0)] {
        let clip = projection * DVec4::new(0.0, 0.0, -distance, 1.0);
        assert_close([clip.z / clip.w], [expected_depth]);
    }
    let half_height = (camera.fov.to_radians() / 2.0).tan();
    let clip = projection * DVec4::new(half_height * camera.aspect_ratio, half_height, -1.0, 1.0);
    assert_close([clip.x / clip.w, clip.y / clip.w], [1.0, 1.0]);
}

#[test]
fn camera_view_accounts_for_its_parent_transform() {
    let parent = Rc::new(RefCell::new(Transform::new()));
    parent.borrow_mut().position = DVec3::new(2.0, -1.0, 3.0);
    let camera = Camera::new(1.0);
    camera.transform_mut().position = DVec3::new(0.0, 0.0, 5.0);
    camera.transform_mut().parent = Some(parent);
    let view = camera.view_matrix();
    assert_close(
        (view * DVec4::new(2.0, -1.0, 8.0, 1.0)).into(),
        [0.0, 0.0, 0.0, 1.0],
    );
    assert_close(
        (view * DVec4::new(2.0, -1.0, 3.0, 1.0)).into(),
        [0.0, 0.0, -5.0, 1.0],
    );
}

#[test]
fn camera_view_uses_its_quaternion_orientation() {
    let camera = Camera::new(1.0);
    camera.transform_mut().position = DVec3::new(2.0, 3.0, 4.0);
    camera.transform_mut().rotation = DQuat::from_rotation_y(FRAC_PI_2);
    let view = camera.view_matrix();
    assert_close(
        (view * DVec4::new(2.0, 3.0, 4.0, 1.0)).into(),
        [0.0, 0.0, 0.0, 1.0],
    );
    assert_close(
        (view * DVec4::new(-3.0, 3.0, 4.0, 1.0)).into(),
        [0.0, 0.0, -5.0, 1.0],
    );
}

#[test]
fn orbit_control_keeps_looking_at_the_target_under_a_parent_transform() {
    let parent = Rc::new(RefCell::new(Transform::new()));
    parent.borrow_mut().position = DVec3::new(2.0, -1.0, 3.0);
    parent.borrow_mut().scaling = DVec3::new(2.0, 3.0, 4.0);
    parent.borrow_mut().rotation = DQuat::from_rotation_y(0.4);
    let target = parent.borrow().compute_world_matrix() * DVec4::new(0.0, 0.0, 0.0, 1.0);
    let mut camera = Camera::new(1.0);
    camera.transform_mut().parent = Some(parent);
    let mut control = OrbitControl::default();
    control.set_radius(2.0);
    camera.control = Some(Box::new(control));

    let inputs = [
        Mouse::default(),
        Mouse {
            left_button_pressed: true,
            delta: [20.0, -10.0],
            ..Mouse::default()
        },
        Mouse {
            scroll_delta: 1.0,
            ..Mouse::default()
        },
    ];
    for mouse in inputs {
        camera.listen_to_control(&mouse, 0.016);
        assert!(camera.transform().rotation.is_normalized());
        let radius = camera.transform().position.length();
        assert_close(
            (camera.view_matrix() * target).into(),
            [0.0, 0.0, -radius, 1.0],
        );
    }
}

#[test]
#[should_panic]
fn singular_transform_cannot_produce_a_normal_matrix() {
    let mut transform = Transform::new();
    transform.scaling = DVec3::new(0.0, 1.0, 1.0);
    transform.compute_normal_matrix();
}

#[test]
#[should_panic]
fn singular_parent_transform_cannot_produce_a_camera_view() {
    let parent = Rc::new(RefCell::new(Transform::new()));
    parent.borrow_mut().scaling = DVec3::new(0.0, 1.0, 1.0);
    let camera = Camera::new(1.0);
    camera.transform_mut().parent = Some(parent);
    camera.view_matrix();
}

#[test]
fn gpu_uniforms_keep_their_layout_and_column_major_matrices() {
    assert_eq!(std::mem::size_of::<TransformUniforms>(), 144);
    assert_eq!(std::mem::offset_of!(TransformUniforms, world_matrix), 16);
    assert_eq!(std::mem::offset_of!(TransformUniforms, normal_matrix), 80);
    assert_eq!(std::mem::size_of::<CameraUniforms>(), 80);
    let mut transform = Transform::new();
    transform.position = DVec3::new(3.0, -2.0, 4.0);
    let mut uniforms = TransformUniforms::default();
    uniforms.update(&transform);
    assert_eq!(uniforms.position, [3.0, -2.0, 4.0]);
    assert_eq!(uniforms.world_matrix[3], [3.0, -2.0, 4.0, 1.0]);

    let camera = Camera::new(1.0);
    camera.transform_mut().position = DVec3::new(0.0, 0.0, 5.0);
    let mut uniforms = CameraUniforms::default();
    uniforms.update(&camera);
    let floats: &[f32] = bytemuck::cast_slice(std::slice::from_ref(&uniforms));
    assert_eq!(&floats[16..19], &[0.0, 0.0, 5.0]);
}

#[test]
fn parented_transforms_and_camera_keep_small_offsets_at_large_coordinates() {
    let origin = 1e12;
    let parent = Rc::new(RefCell::new(Transform::new()));
    parent.borrow_mut().position = DVec3::new(origin, -origin, origin);
    let mut child = Transform::new();
    child.parent = Some(parent.clone());
    child.position = DVec3::new(0.125, 0.25, -0.5);
    let world_position = child.compute_world_matrix().transform_point3(DVec3::ZERO);
    assert_eq!(
        world_position,
        DVec3::new(origin + 0.125, -origin + 0.25, origin - 0.5)
    );

    let camera = Camera::new(1.0);
    camera.transform_mut().parent = Some(parent);
    let relative_position = camera.view_matrix().transform_point3(world_position);
    assert_close(relative_position.to_array(), [0.125, 0.25, -0.5]);
}

#[test]
fn normals_and_tangents_follow_geometry_and_uvs() {
    let origin = 10.0;
    let mut buffers = VertexData {
        positions: vec![
            [origin, 0.0, origin],
            [origin, 0.0, origin + 0.25],
            [origin + 0.5, 0.0, origin],
        ],
        uvs: vec![[0.0, 0.0], [0.0, 1.0], [1.0, 0.0]],
        indices: vec![0, 1, 2],
        ..VertexData::default()
    };
    buffers.create_normals();
    buffers.create_tangents();
    for normal in &buffers.normals {
        assert_eq!(*normal, [0.0, 1.0, 0.0]);
    }
    for tangent in &buffers.tangents {
        assert_eq!(*tangent, [1.0, 0.0, 0.0]);
    }
}

#[test]
fn gpu_vertex_layout_and_attributes_are_preserved() {
    let vertices = VertexData {
        positions: vec![[1.25, 2.0, 3.0]],
        colors: vec![[1.0, 0.5, 0.25]],
        normals: vec![[0.0, 1.0, 0.0]],
        tangents: vec![[1.0, 0.0, 0.0]],
        uvs: vec![[0.25, 0.5]],
        ..VertexData::default()
    };
    let gpu_vertices = zip_vertex_data(&vertices);
    assert_eq!(std::mem::size_of::<Vertex>(), 56);
    assert_eq!(std::mem::offset_of!(Vertex, uv), 48);
    assert_eq!(gpu_vertices[0].position, [1.25, 2.0, 3.0]);
    assert_eq!(gpu_vertices[0].color, [1.0, 0.5, 0.25]);
    assert_eq!(gpu_vertices[0].normal, [0.0, 1.0, 0.0]);
    assert_eq!(gpu_vertices[0].tangent, [1.0, 0.0, 0.0]);
    assert_eq!(gpu_vertices[0].uv, [0.25, 0.5]);
    assert_eq!(vertices.positions[0], gpu_vertices[0].position);
}

#[test]
fn transform_uniforms_narrow_without_changing_cpu_state() {
    let precise_value = 1.0 + 2.0_f64.powi(-30);
    let mut transform = Transform::new();
    transform.position = DVec3::new(precise_value, 0.0, 0.0);
    let mut uniforms = TransformUniforms::default();
    uniforms.update(&transform);
    assert_eq!(uniforms.position, [1.0, 0.0, 0.0]);
    assert_eq!(uniforms.world_matrix[3], [1.0, 0.0, 0.0, 1.0]);
    assert_eq!(transform.position.x, precise_value);
}

#[test]
fn light_uniforms_preserve_lighting_values_and_narrow_only_positions() {
    let precise_position = 1.0 + 2.0_f64.powi(-30);
    let intensity: f32 = 1.0 / 3.0;
    let mut point = PointLight::default();
    point.set_color(1.0, 0.5, 0.25);
    point.set_intensity(intensity);
    point.transform_mut().position = DVec3::new(precise_position, 2.0, 3.0);
    let mut point_uniforms = PointLightUniforms::default();
    point_uniforms.update(&point);
    assert_eq!(std::mem::size_of::<PointLightUniforms>(), 32);
    let floats: &[f32] = bytemuck::cast_slice(std::slice::from_ref(&point_uniforms));
    assert_eq!(&floats[..3], &[1.0, 0.5, 0.25]);
    assert_eq!(&floats[4..], &[1.0, 2.0, 3.0, intensity]);
    assert_eq!(point.color(), [1.0, 0.5, 0.25]);
    assert_eq!(point.intensity(), intensity);
    assert_eq!(point.transform().position.x, precise_position);

    let mut directional = DirectionalLight::default();
    directional.set_color(1.0, 0.5, 0.25);
    directional.set_intensity(intensity);
    let mut directional_uniforms = DirectionalLightUniform::default();
    directional_uniforms.update(&directional);
    assert_eq!(std::mem::size_of::<DirectionalLightUniform>(), 32);
    let floats: &[f32] = bytemuck::cast_slice(std::slice::from_ref(&directional_uniforms));
    assert_eq!(&floats[..3], &[1.0, 0.5, 0.25]);
    assert_eq!(&floats[4..7], &[0.707, -0.707, 0.0]);
    assert_eq!(floats[7], intensity);
    assert_eq!(directional.color(), [1.0, 0.5, 0.25]);
    assert_eq!(directional.intensity(), intensity);
}
