use gilgamesh::camera::Camera;
use gilgamesh::camera::uniforms::CameraUniforms;
use gilgamesh::geometry::cpu_mesh_buffers::CpuMeshBuffers;
use gilgamesh::geometry::cpu_vertex_buffers::CpuVertexBuffers;
use gilgamesh::geometry::gpu_mesh_buffers::{Vertex, to_gpu_vertices};
use gilgamesh::input::mouse::Mouse;
use gilgamesh::input::orbit_control::OrbitControl;
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
    assert_close((transform.world_matrix() * point).into(), point.into());
}

#[test]
fn world_transform_applies_rotation_then_translation() {
    let mut transform = Transform::new();
    transform.position = DVec3::new(3.0, -2.0, 4.0);
    transform.rotation = DQuat::from_rotation_z(FRAC_PI_2) * DQuat::from_rotation_x(FRAC_PI_2);
    let position = transform.world_matrix() * DVec4::new(1.0, 2.0, 3.0, 1.0);
    assert_close(position.into(), [6.0, -1.0, 6.0, 1.0]);
}

#[test]
fn child_transform_follows_changes_to_its_parent() {
    let parent = Rc::new(RefCell::new(Transform::new()));
    parent.borrow_mut().position = DVec3::new(5.0, -4.0, 2.0);
    parent.borrow_mut().rotation = DQuat::from_rotation_z(FRAC_PI_2);
    let mut child = Transform::new();
    child.parent = Some(parent.clone());
    child.position = DVec3::new(1.0, 2.0, 3.0);
    let position = child.world_matrix() * DVec4::new(0.0, 0.0, 0.0, 1.0);
    assert_close(position.into(), [3.0, -3.0, 5.0, 1.0]);

    parent.borrow_mut().position = DVec3::new(8.0, -4.0, 2.0);
    let position = child.world_matrix() * DVec4::new(0.0, 0.0, 0.0, 1.0);
    assert_close(position.into(), [6.0, -3.0, 5.0, 1.0]);
}

#[test]
fn rigid_hierarchy_preserves_lengths_and_perpendicular_normals_in_gpu_uniforms() {
    let parent = Rc::new(RefCell::new(Transform::new()));
    parent.borrow_mut().position = DVec3::new(5.0, -4.0, 2.0);
    parent.borrow_mut().rotation = DQuat::from_rotation_z(0.4);
    let mut child = Transform::new();
    child.parent = Some(parent);
    child.position = DVec3::new(1.0, 2.0, 3.0);
    child.rotation = DQuat::from_rotation_x(0.7);

    let mut uniforms = TransformUniforms::default();
    uniforms.update(&child);
    let world = glam::Mat4::from_cols_array_2d(&uniforms.world_matrix);
    let normal = world.transform_vector3(glam::Vec3::Y);
    let tangent = world.transform_point3(glam::Vec3::X) - world.transform_point3(glam::Vec3::ZERO);
    assert!((normal.length() - 1.0).abs() < 1e-6);
    assert!((tangent.length() - 1.0).abs() < 1e-6);
    assert!(normal.dot(tangent).abs() < 1e-6);
}

#[test]
fn local_axes_only_depend_on_rotation_and_stay_normalized() {
    let mut transform = Transform::new();
    transform.position = DVec3::new(100.0, -20.0, 50.0);
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
fn camera_view_accounts_for_its_parent_transform() {
    let parent = Rc::new(RefCell::new(Transform::new()));
    parent.borrow_mut().position = DVec3::new(2.0, -1.0, 3.0);
    let camera: Camera = Camera::default();
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
    let camera: Camera = Camera::default();
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
    parent.borrow_mut().rotation = DQuat::from_rotation_y(0.4);
    let target = parent.borrow().world_matrix() * DVec4::new(0.0, 0.0, 0.0, 1.0);
    let mut camera: Camera = Camera::default();
    camera.transform_mut().parent = Some(parent);
    let mut control = OrbitControl::default();
    control.set_radius(2.0);

    let inputs = [
        Mouse::default(),
        Mouse {
            left_button_pressed: true,
            delta: [20.0, -10.0],
            ..Mouse::default()
        },
        Mouse {
            scroll_delta_lines: 1.0,
            ..Mouse::default()
        },
    ];
    for mouse in inputs {
        control.update(&mut camera, &mouse, 0.016);
        assert!(camera.transform().rotation.is_normalized());
        let radius = camera.transform().position.length();
        assert_close(
            (camera.view_matrix() * target).into(),
            [0.0, 0.0, -radius, 1.0],
        );
    }
}

#[test]
fn gpu_uniforms_keep_their_layout_and_column_major_matrices() {
    assert_eq!(std::mem::size_of::<TransformUniforms>(), 80);
    assert_eq!(std::mem::offset_of!(TransformUniforms, world_matrix), 16);
    assert_eq!(std::mem::size_of::<CameraUniforms>(), 80);
    let mut transform = Transform::new();
    transform.position = DVec3::new(3.0, -2.0, 4.0);
    let mut uniforms = TransformUniforms::default();
    uniforms.update(&transform);
    assert_eq!(uniforms.position, [3.0, -2.0, 4.0]);
    assert_eq!(uniforms.world_matrix[3], [3.0, -2.0, 4.0, 1.0]);

    let camera: Camera = Camera::default();
    camera.transform_mut().position = DVec3::new(0.0, 0.0, 5.0);
    let mut uniforms = CameraUniforms::default();
    uniforms.update(&camera);
    let floats: &[f32] = bytemuck::cast_slice(std::slice::from_ref(&uniforms));
    assert_eq!(&floats[16..19], &[0.0, 0.0, 5.0]);
}

#[test]
fn shader_positions_follow_parent_translation_and_rotation() {
    let grandparent = Rc::new(RefCell::new(Transform::new()));
    grandparent.borrow_mut().position = DVec3::new(100.0, 0.0, 0.0);
    let parent = Rc::new(RefCell::new(Transform::new()));
    {
        let mut transform = parent.borrow_mut();
        transform.parent = Some(grandparent);
        transform.rotation = DQuat::from_rotation_z(FRAC_PI_2);
    }
    let camera: Camera = Camera::default();
    let light = PointLight::default();
    for mut transform in [camera.transform_mut(), light.transform_mut()] {
        transform.parent = Some(parent.clone());
        transform.position = DVec3::new(1.0, 2.0, 3.0);
    }

    for (translation, expected) in [
        (DVec3::new(10.0, 20.0, 30.0), [108.0, 21.0, 33.0]),
        (DVec3::new(30.0, 40.0, 50.0), [128.0, 41.0, 53.0]),
    ] {
        parent.borrow_mut().position = translation;
        let mut camera_uniforms = CameraUniforms::default();
        camera_uniforms.update(&camera);
        let floats: &[f32] = bytemuck::cast_slice(std::slice::from_ref(&camera_uniforms));
        assert_eq!(&floats[16..19], &expected);

        let mut light_uniforms = PointLightUniforms::default();
        light_uniforms.update(&light);
        let floats: &[f32] = bytemuck::cast_slice(std::slice::from_ref(&light_uniforms));
        assert_eq!(&floats[4..7], &expected);

        let mut object_uniforms = TransformUniforms::default();
        object_uniforms.update(&camera.transform());
        assert_eq!(object_uniforms.position, expected);
        assert_eq!(&object_uniforms.world_matrix[3][..3], &expected);
    }
}

#[test]
fn parented_transforms_and_camera_keep_small_offsets_at_large_coordinates() {
    let origin = 1e12;
    let parent = Rc::new(RefCell::new(Transform::new()));
    parent.borrow_mut().position = DVec3::new(origin, -origin, origin);
    let mut child = Transform::new();
    child.parent = Some(parent.clone());
    child.position = DVec3::new(0.125, 0.25, -0.5);
    let world_position = child.world_matrix().transform_point3(DVec3::ZERO);
    assert_eq!(
        world_position,
        DVec3::new(origin + 0.125, -origin + 0.25, origin - 0.5)
    );

    let camera: Camera = Camera::default();
    camera.transform_mut().parent = Some(parent);
    let relative_position = camera.view_matrix().transform_point3(world_position);
    assert_close(relative_position.to_array(), [0.125, 0.25, -0.5]);
}

#[test]
fn normals_and_tangents_follow_geometry_and_uvs() {
    let origin = 10.0;
    let mut buffers = CpuMeshBuffers {
        vertices: CpuVertexBuffers {
            positions: vec![
                [origin, 0.0, origin],
                [origin, 0.0, origin + 0.25],
                [origin + 0.5, 0.0, origin],
            ],
            uvs: vec![[0.0, 0.0], [0.0, 1.0], [1.0, 0.0]],
            ..CpuVertexBuffers::default()
        },
        indices: vec![0, 1, 2],
    };
    buffers.create_normals();
    buffers.create_tangents();
    for normal in &buffers.vertices.normals {
        assert_eq!(*normal, [0.0, 1.0, 0.0]);
    }
    for tangent in &buffers.vertices.tangents {
        assert_eq!(*tangent, [1.0, 0.0, 0.0]);
    }
}

#[test]
fn degenerate_uvs_do_not_poison_tangents_on_adjacent_triangles() {
    let mut buffers = CpuMeshBuffers {
        vertices: CpuVertexBuffers {
            positions: vec![
                [0.0, 0.0, 0.0],
                [0.0, 0.0, 1.0],
                [1.0, 0.0, 1.0],
                [1.0, 0.0, 0.0],
            ],
            normals: vec![[0.0, 1.0, 0.0]; 4],
            uvs: vec![[0.0, 0.0], [0.0, 0.0], [0.0, 1.0], [1.0, 0.0]],
            ..CpuVertexBuffers::default()
        },
        indices: vec![0, 1, 2, 0, 2, 3],
    };
    buffers.create_tangents();
    for tangent in &buffers.vertices.tangents {
        let tangent = glam::Vec3::from_array(*tangent);
        assert!(tangent.is_finite());
        assert!((tangent.length() - 1.0).abs() < 1e-6);
        assert!(tangent.dot(glam::Vec3::Y).abs() < 1e-6);
    }
    for index in [0, 2, 3] {
        assert_eq!(buffers.vertices.tangents[index], [1.0, 0.0, 0.0]);
    }
}

#[test]
fn gpu_vertex_layout_and_attributes_are_preserved() {
    let vertices = CpuVertexBuffers {
        positions: vec![[1.25, 2.0, 3.0]],
        colors: vec![[1.0, 0.5, 0.25]],
        normals: vec![[0.0, 1.0, 0.0]],
        tangents: vec![[1.0, 0.0, 0.0]],
        uvs: vec![[0.25, 0.5]],
    };
    let gpu_vertices = to_gpu_vertices(&vertices);
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
