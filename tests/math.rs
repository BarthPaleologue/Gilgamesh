use gilgamesh::camera::Camera;
use gilgamesh::camera::uniforms::CameraUniforms;
use gilgamesh::transform::{Transform, TransformUniforms, Transformable};
use glam::{Vec3, Vec4};
use std::cell::RefCell;
use std::f32::consts::FRAC_PI_2;
use std::rc::Rc;

fn assert_close<const N: usize>(actual: [f32; N], expected: [f32; N]) {
    for (actual, expected) in actual.into_iter().zip(expected) {
        assert!(
            (actual - expected).abs() < 1e-5,
            "expected {expected}, got {actual}"
        );
    }
}

#[test]
fn world_transform_keeps_rotation_then_nonuniform_scale_then_translation() {
    let mut transform = Transform::new();
    transform.set_position(3.0, -2.0, 4.0);
    transform.set_scaling(2.0, 3.0, 4.0);
    transform.set_rotation(FRAC_PI_2, 0.0, FRAC_PI_2);
    let position = transform.compute_world_matrix() * Vec4::new(1.0, 2.0, 3.0, 1.0);
    assert_close(position.into(), [9.0, 1.0, 12.0, 1.0]);
}

#[test]
fn child_transform_follows_changes_to_its_parent() {
    let parent = Rc::new(RefCell::new(Transform::new()));
    parent.borrow_mut().set_position(5.0, -4.0, 2.0);
    parent.borrow_mut().set_scaling(2.0, 3.0, 4.0);
    parent.borrow_mut().set_rotation(0.0, 0.0, FRAC_PI_2);
    let mut child = Transform::new();
    child.parent = Some(parent.clone());
    child.set_position(1.0, 2.0, 3.0);
    let position = child.compute_world_matrix() * Vec4::new(0.0, 0.0, 0.0, 1.0);
    assert_close(position.into(), [1.0, -1.0, 14.0, 1.0]);

    parent.borrow_mut().set_position(8.0, -4.0, 2.0);
    let position = child.compute_world_matrix() * Vec4::new(0.0, 0.0, 0.0, 1.0);
    assert_close(position.into(), [4.0, -1.0, 14.0, 1.0]);
}

#[test]
fn normal_transform_accounts_for_nonuniform_scale_and_rotation() {
    let mut transform = Transform::new();
    transform.set_scaling(2.0, 4.0, 8.0);
    transform.set_rotation(0.0, 0.0, FRAC_PI_2);
    let normal = transform.compute_normal_matrix() * Vec4::new(1.0, 0.0, 0.0, 0.0);
    assert_close(normal.into(), [0.0, 0.25, 0.0, 0.0]);
}

#[test]
fn local_axes_only_depend_on_rotation_and_stay_normalized() {
    let mut transform = Transform::new();
    transform.set_position(100.0, -20.0, 50.0);
    transform.set_scaling(2.0, 3.0, 4.0);
    transform.set_rotation(0.0, FRAC_PI_2, 0.0);
    assert_close(transform.forward().into(), [1.0, 0.0, 0.0]);
    assert_close(transform.right().into(), [0.0, 0.0, -1.0]);
    assert_close(transform.up().into(), [0.0, 1.0, 0.0]);
    assert_close(
        transform
            .local_direction_to_world(Vec3::new(0.0, 0.0, 5.0))
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
        let clip = projection * Vec4::new(0.0, 0.0, -distance, 1.0);
        assert_close([clip.z / clip.w], [expected_depth]);
    }
    let half_height = (camera.fov.to_radians() / 2.0).tan();
    let clip = projection * Vec4::new(half_height * camera.aspect_ratio, half_height, -1.0, 1.0);
    assert_close([clip.x / clip.w, clip.y / clip.w], [1.0, 1.0]);
}

#[test]
fn camera_view_accounts_for_its_parent_transform() {
    let parent = Rc::new(RefCell::new(Transform::new()));
    parent.borrow_mut().set_position(2.0, -1.0, 3.0);
    let camera = Camera::new(1.0);
    camera.transform_mut().set_position(0.0, 0.0, 5.0);
    camera.transform_mut().parent = Some(parent);
    let view = camera.view_matrix();
    assert_close(
        (view * Vec4::new(2.0, -1.0, 8.0, 1.0)).into(),
        [0.0, 0.0, 0.0, 1.0],
    );
    assert_close(
        (view * Vec4::new(2.0, -1.0, 3.0, 1.0)).into(),
        [0.0, 0.0, -5.0, 1.0],
    );
}

#[test]
#[should_panic]
fn singular_transform_cannot_produce_a_normal_matrix() {
    let mut transform = Transform::new();
    transform.set_scaling(0.0, 1.0, 1.0);
    transform.compute_normal_matrix();
}

#[test]
#[should_panic]
fn singular_parent_transform_cannot_produce_a_camera_view() {
    let parent = Rc::new(RefCell::new(Transform::new()));
    parent.borrow_mut().set_scaling(0.0, 1.0, 1.0);
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
    transform.set_position(3.0, -2.0, 4.0);
    let mut uniforms = TransformUniforms::default();
    uniforms.update(&transform);
    assert_eq!(uniforms.position, [3.0, -2.0, 4.0]);
    assert_eq!(uniforms.world_matrix[3], [3.0, -2.0, 4.0, 1.0]);

    let camera = Camera::new(1.0);
    camera.transform_mut().set_position(0.0, 0.0, 5.0);
    let mut uniforms = CameraUniforms::default();
    uniforms.update(&camera);
    let floats: &[f32] = bytemuck::cast_slice(std::slice::from_ref(&uniforms));
    assert_eq!(&floats[16..19], &[0.0, 0.0, 5.0]);
}
