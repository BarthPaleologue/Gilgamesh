use crate::camera::uniforms::CameraUniforms;
use crate::camera::{Camera, CameraProjection};
use crate::lights::directional_light::DirectionalLight;
use crate::lights::point_light::PointLight;
use crate::settings::MAX_POINT_LIGHTS;

/// Captures camera uniforms while borrowing the application's lights for a render pass.
pub struct FrameBindingsData<'a> {
    pub camera: CameraUniforms,
    pub(crate) directional_light: &'a DirectionalLight,
    pub(crate) point_lights: &'a [PointLight],
}

impl<'a> FrameBindingsData<'a> {
    /// # Panics
    /// Panics if more than [`MAX_POINT_LIGHTS`] lights are supplied.
    pub fn new(
        camera: &Camera<impl CameraProjection>,
        directional_light: &'a DirectionalLight,
        point_lights: &'a [PointLight],
    ) -> Self {
        assert!(
            point_lights.len() <= MAX_POINT_LIGHTS,
            "A render context supports at most {MAX_POINT_LIGHTS} point lights, got {}.",
            point_lights.len(),
        );
        let mut camera_uniforms = CameraUniforms::default();
        camera_uniforms.update(camera);
        Self {
            camera: camera_uniforms,
            directional_light,
            point_lights,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::camera::{Orthographic, Perspective};
    use crate::lights::light::Light;
    use crate::transform::Transformable;
    use glam::{DMat4, DVec3};

    #[test]
    fn borrows_the_app_lights() {
        let camera: Camera = Camera::default();
        let directional_light = DirectionalLight::default();
        let mut point_lights = [PointLight::default()];
        point_lights[0].set_intensity(0.2);

        let context = FrameBindingsData::new(&camera, &directional_light, &point_lights);
        assert!(std::ptr::eq(
            context.directional_light,
            &raw const directional_light
        ));
        assert!(std::ptr::eq(
            context.point_lights.as_ptr(),
            point_lights.as_ptr()
        ));
        assert_eq!(context.point_lights[0].intensity(), 0.2);
    }

    #[test]
    fn captures_camera_matrix_and_position_before_later_changes() {
        let mut camera: Camera<Perspective> = Camera::default();
        camera.transform_mut().position = DVec3::new(3.0, 4.0, 5.0);
        let matrix = camera.projection_view().as_mat4().to_cols_array();
        let light = DirectionalLight::default();
        let context = FrameBindingsData::new(&camera, &light, &[]);

        camera.projection.aspect_ratio = 2.0;
        camera.transform_mut().position = DVec3::new(6.0, 7.0, 8.0);
        let next = FrameBindingsData::new(&camera, &light, &[]);
        drop(camera);

        let captured: [f32; 20] = bytemuck::cast(context.camera);
        assert_eq!(captured[..16], matrix);
        assert_eq!(captured[16..19], [3.0, 4.0, 5.0]);
        assert_ne!(
            bytemuck::bytes_of(&context.camera),
            bytemuck::bytes_of(&next.camera)
        );
    }

    #[test]
    fn different_projection_types_produce_the_same_bindings_type() {
        struct CustomProjection(Perspective);
        impl CameraProjection for CustomProjection {
            fn matrix(&self) -> DMat4 {
                self.0.matrix()
            }
        }

        let perspective: Camera<Perspective> = Camera::default();
        let orthographic = Camera::new(Orthographic {
            size_vertical: 4.0,
            aspect_ratio: 1.5,
            near: 0.1,
            far: 100.0,
        });
        let custom = Camera::new(CustomProjection(Perspective {
            fov_vertical: 1.0,
            ..Default::default()
        }));
        let light = DirectionalLight::default();
        let frames = [
            FrameBindingsData::new(&perspective, &light, &[]),
            FrameBindingsData::new(&orthographic, &light, &[]),
            FrameBindingsData::new(&custom, &light, &[]),
        ];
        let matrices = [
            perspective.projection_view(),
            orthographic.projection_view(),
            custom.projection_view(),
        ];
        for (frame, matrix) in frames.into_iter().zip(matrices) {
            let camera: [f32; 20] = bytemuck::cast(frame.camera);
            assert_eq!(camera[..16], matrix.as_mat4().to_cols_array());
        }
    }

    #[test]
    fn accepts_no_point_lights() {
        let camera: Camera = Camera::default();
        let light = DirectionalLight::default();
        let context = FrameBindingsData::new(&camera, &light, &[]);
        assert!(context.point_lights.is_empty());
    }

    #[test]
    fn accepts_the_maximum_point_light_count() {
        let camera: Camera = Camera::default();
        let light = DirectionalLight::default();
        let point_lights = std::array::from_fn::<_, MAX_POINT_LIGHTS, _>(|_| PointLight::default());
        let context = FrameBindingsData::new(&camera, &light, &point_lights);
        assert_eq!(context.point_lights.len(), MAX_POINT_LIGHTS);
    }

    #[test]
    #[should_panic(expected = "A render context supports at most")]
    fn rejects_more_point_lights_than_material_buffers_support() {
        let camera: Camera = Camera::default();
        let light = DirectionalLight::default();
        let point_lights =
            std::array::from_fn::<_, { MAX_POINT_LIGHTS + 1 }, _>(|_| PointLight::default());
        FrameBindingsData::new(&camera, &light, &point_lights);
    }
}
