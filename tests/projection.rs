use gilgamesh::camera::uniforms::CameraUniforms;
use gilgamesh::camera::{Camera, CameraProjection, Orthographic, Perspective};
use glam::{DMat4, DVec2, DVec3, Vec3, dcamera::rh::proj::directx};
use std::f64::consts::FRAC_PI_3;

fn assert_projected(matrix: DMat4, point: DVec3, expected: DVec3) {
    let actual = matrix.project_point3(point);
    assert!(
        (actual - expected).length() < 1e-12,
        "expected {expected}, got {actual}"
    );
}

#[test]
fn perspective_uses_radians_and_reverse_depth() {
    let near = 0.1;
    let aspect_ratio = 1.5;
    let camera = Camera::new(Perspective {
        fov_vertical: FRAC_PI_3,
        near,
        ..Default::default()
    });
    let projection = camera.projection_matrix(aspect_ratio);
    for distance in [near, 2.0 * near, 1e12] {
        assert_projected(
            projection,
            DVec3::new(0.0, 0.0, -distance),
            DVec3::new(0.0, 0.0, near / distance),
        );
        assert!(projection.project_point3(DVec3::new(0.0, 0.0, -distance)).z > 0.0);
    }
    let half_height = (FRAC_PI_3 / 2.0).tan();
    let half_width = half_height * aspect_ratio;
    let corner = projection.project_point3(DVec3::new(half_width, half_height, -1.0));
    assert!((corner.x - 1.0).abs() < 1e-12);
    assert!((corner.y - 1.0).abs() < 1e-12);
}

#[test]
fn default_camera_preserves_distant_depths_after_gpu_conversion() {
    let camera: Camera = Camera::default();
    let matrix = camera.projection_matrix(1.0).as_mat4();
    let nearer = matrix.project_point3(Vec3::new(0.0, 0.0, -100_000_000.0)).z;
    let farther = matrix.project_point3(Vec3::new(0.0, 0.0, -100_001_024.0)).z;
    assert!(0.0 < farther && farther < nearer && nearer < 1.0);
}

#[test]
fn orthographic_keeps_visible_size_at_every_depth_and_tracks_parameter_changes() {
    let near = 0.5;
    let far = 100.0;
    let mut camera = Camera::new(Orthographic {
        size_vertical: 4.0,
        near,
        far,
    });
    let projection = camera.projection_matrix(1.5);
    for (distance, depth) in [(near, 1.0), (far, 0.0)] {
        for (x, expected_x) in [(-3.0, -1.0), (3.0, 1.0)] {
            for (y, expected_y) in [(-2.0, -1.0), (2.0, 1.0)] {
                assert_projected(
                    projection,
                    DVec3::new(x, y, -distance),
                    DVec3::new(expected_x, expected_y, depth),
                );
            }
        }
    }
    camera.projection.size_vertical = 8.0;
    assert_projected(
        camera.projection_matrix(2.0),
        DVec3::new(8.0, 4.0, -near),
        DVec3::new(1.0, 1.0, 1.0),
    );
}

#[test]
fn shifted_perspective_preserves_signed_angular_frusta() {
    let near = 0.25;
    for [angle_left, angle_right, angle_bottom, angle_top] in
        [[-0.7_f64, 0.4, -0.3, 0.6], [0.1, 0.6, -0.4, -0.1]]
    {
        let [left, right, bottom, top] =
            [angle_left, angle_right, angle_bottom, angle_top].map(f64::tan);
        let width = right - left;
        let height = top - bottom;
        let projection = Perspective {
            fov_vertical: 2.0 * (height / 2.0).atan(),
            frustum_offset: DVec2::new((right + left) / width, (top + bottom) / height),
            near,
        };
        let matrix = projection.matrix(width / height);
        for distance in [near, 2.0 * near, 1e12] {
            for (x, expected_x) in [(left, -1.0), (right, 1.0)] {
                for (y, expected_y) in [(bottom, -1.0), (top, 1.0)] {
                    assert_projected(
                        matrix,
                        DVec3::new(distance * x, distance * y, -distance),
                        DVec3::new(expected_x, expected_y, near / distance),
                    );
                }
            }
        }
        assert!(matrix.project_point3(DVec3::new(0.0, 0.0, -1e12)).z > 0.0);
    }
}

#[test]
fn centered_perspective_matches_glam() {
    let projection = Perspective {
        fov_vertical: FRAC_PI_3,
        ..Default::default()
    };
    assert_eq!(
        projection.matrix(1.5),
        directx::perspective_infinite_reverse(FRAC_PI_3, 1.5, projection.near)
    );
}

#[test]
fn render_aspect_changes_width_preserving_shift_and_height() {
    let projection = Perspective {
        fov_vertical: FRAC_PI_3,
        frustum_offset: DVec2::new(0.5, -1.5),
        near: 0.25,
    };
    let half_height = (FRAC_PI_3 / 2.0).tan();
    for aspect_ratio in [0.5, 1.0, 2.0] {
        let matrix = projection.matrix(aspect_ratio);
        for (point, expected) in [
            (
                DVec3::new(1.5 * half_height * aspect_ratio, -0.5 * half_height, -1.0),
                DVec3::new(1.0, 1.0, 0.25),
            ),
            (
                DVec3::new(-0.5 * half_height * aspect_ratio, -2.5 * half_height, -1.0),
                DVec3::new(-1.0, -1.0, 0.25),
            ),
        ] {
            assert_projected(matrix, point, expected);
        }
    }
}

fn camera_uniforms<ProjectionType: CameraProjection>(
    camera: &Camera<ProjectionType>,
    aspect_ratio: f64,
) -> CameraUniforms {
    let mut uniforms = CameraUniforms::default();
    uniforms.update(camera, aspect_ratio);
    uniforms
}

#[test]
fn typed_and_runtime_projection_cameras_share_the_uniform_interface() {
    let perspective: Camera<Perspective> = Camera::default();
    let orthographic = Camera::new(Orthographic {
        size_vertical: 4.0,
        near: 0.5,
        far: 100.0,
    });
    let shifted = Camera::new(Perspective {
        frustum_offset: DVec2::new(-0.25, 0.5),
        ..Default::default()
    });
    for aspect_ratio in [0.5, 1.5, 2.0] {
        let cameras = [
            (
                camera_uniforms(&perspective, aspect_ratio),
                perspective.projection.into(),
            ),
            (
                camera_uniforms(&orthographic, aspect_ratio),
                orthographic.projection.into(),
            ),
            (
                camera_uniforms(&shifted, aspect_ratio),
                shifted.projection.into(),
            ),
        ];
        let mut runtime_camera: Camera = Camera::default();
        for (typed_uniforms, projection) in cameras {
            runtime_camera.projection = projection;
            let runtime_uniforms = camera_uniforms(&runtime_camera, aspect_ratio);
            assert_eq!(
                bytemuck::bytes_of(&typed_uniforms),
                bytemuck::bytes_of(&runtime_uniforms)
            );
        }
    }
}
