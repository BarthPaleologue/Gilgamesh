use gilgamesh::camera::uniforms::CameraUniforms;
use gilgamesh::camera::{Camera, CameraProjection, OffAxis, Orthographic, Perspective};
use glam::{DMat4, DVec3, Vec3};
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
    let far = 10.0;
    let aspect_ratio = 1.5;
    let camera = Camera::new(Perspective {
        fov_vertical: FRAC_PI_3,
        aspect_ratio,
        near,
        far,
    });
    let projection = camera.projection_matrix();
    for (distance, depth) in [(near, 1.0), (far, 0.0)] {
        assert_projected(
            projection,
            DVec3::new(0.0, 0.0, -distance),
            DVec3::new(0.0, 0.0, depth),
        );
    }
    let half_height = (FRAC_PI_3 / 2.0).tan();
    let half_width = half_height * aspect_ratio;
    let corner = projection.project_point3(DVec3::new(half_width, half_height, -1.0));
    assert!((corner.x - 1.0).abs() < 1e-12);
    assert!((corner.y - 1.0).abs() < 1e-12);
}

#[test]
fn perspective_preserves_distant_depths_after_gpu_conversion() {
    let matrix = Perspective {
        far: 1e12,
        ..Default::default()
    }
    .matrix()
    .as_mat4();
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
        aspect_ratio: 1.5,
        near,
        far,
    });
    let projection = camera.projection_matrix();
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
    camera.projection.aspect_ratio = 2.0;
    assert_projected(
        camera.projection_matrix(),
        DVec3::new(8.0, 4.0, -near),
        DVec3::new(1.0, 1.0, 1.0),
    );
}

#[test]
fn off_axis_maps_signed_angular_bounds_to_clip_edges() {
    let near = 0.25;
    let far = 500.0;
    for [angle_left, angle_right, angle_bottom, angle_top] in
        [[-0.7_f64, 0.4, -0.3, 0.6], [0.1, 0.6, -0.4, -0.1]]
    {
        let projection = OffAxis {
            angle_left,
            angle_right,
            angle_bottom,
            angle_top,
            near,
            far,
        };
        let matrix = projection.matrix();
        for (distance, depth) in [(near, 1.0), (far, 0.0)] {
            for (angle_x, expected_x) in [(angle_left, -1.0), (angle_right, 1.0)] {
                for (angle_y, expected_y) in [(angle_bottom, -1.0), (angle_top, 1.0)] {
                    assert_projected(
                        matrix,
                        DVec3::new(
                            distance * angle_x.tan(),
                            distance * angle_y.tan(),
                            -distance,
                        ),
                        DVec3::new(expected_x, expected_y, depth),
                    );
                }
            }
        }
    }
}

#[test]
fn symmetric_off_axis_matches_perspective() {
    let aspect_ratio = 1.5;
    let vertical_angle = FRAC_PI_3 / 2.0;
    let horizontal_angle = (vertical_angle.tan() * aspect_ratio).atan();
    let perspective = Perspective {
        fov_vertical: FRAC_PI_3,
        aspect_ratio,
        near: 0.1,
        far: 100.0,
    }
    .matrix();
    let off_axis = OffAxis {
        angle_left: -horizontal_angle,
        angle_right: horizontal_angle,
        angle_bottom: -vertical_angle,
        angle_top: vertical_angle,
        near: 0.1,
        far: 100.0,
    }
    .matrix();
    for (actual, expected) in off_axis
        .to_cols_array()
        .into_iter()
        .zip(perspective.to_cols_array())
    {
        assert!((actual - expected).abs() < 1e-12);
    }
}

fn camera_uniforms<ProjectionType: CameraProjection>(
    camera: &Camera<ProjectionType>,
) -> CameraUniforms {
    let mut uniforms = CameraUniforms::default();
    uniforms.update(camera);
    uniforms
}

#[test]
fn typed_and_runtime_projection_cameras_share_the_uniform_interface() {
    let perspective: Camera<Perspective> = Camera::default();
    let orthographic = Camera::new(Orthographic {
        size_vertical: 4.0,
        aspect_ratio: 1.5,
        near: 0.5,
        far: 100.0,
    });
    let off_axis = Camera::new(OffAxis {
        angle_left: -0.7,
        angle_right: 0.4,
        angle_bottom: -0.3,
        angle_top: 0.6,
        near: 0.25,
        far: 500.0,
    });
    let cameras = [
        (camera_uniforms(&perspective), perspective.projection.into()),
        (
            camera_uniforms(&orthographic),
            orthographic.projection.into(),
        ),
        (camera_uniforms(&off_axis), off_axis.projection.into()),
    ];
    let mut runtime_camera: Camera = Camera::default();
    for (typed_uniforms, projection) in cameras {
        runtime_camera.projection = projection;
        let runtime_uniforms = camera_uniforms(&runtime_camera);
        assert_eq!(
            bytemuck::bytes_of(&typed_uniforms),
            bytemuck::bytes_of(&runtime_uniforms)
        );
    }
}
