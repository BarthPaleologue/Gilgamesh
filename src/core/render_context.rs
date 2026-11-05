use crate::camera::uniforms::CameraUniforms;
use crate::camera::{Camera, CameraProjection, Exposure};
use crate::lights::directional_light::DirectionalLight;
use crate::lights::point_light::PointLight;
use crate::settings::{MAX_DIRECTIONAL_LIGHTS, MAX_POINT_LIGHTS};

/// Borrows the application's camera and lights for a render pass.
/// Camera uniforms are resolved using the rendered view's aspect ratio.
pub struct FrameBindingsData<'a> {
    camera_uniforms: Box<dyn Fn(f64) -> CameraUniforms + 'a>,
    pub(crate) exposure: Exposure,
    pub(crate) directional_lights: &'a [DirectionalLight],
    pub(crate) point_lights: &'a [PointLight],
}

impl<'a> FrameBindingsData<'a> {
    /// # Panics
    /// Panics if either slice exceeds [`MAX_DIRECTIONAL_LIGHTS`] or [`MAX_POINT_LIGHTS`].
    pub fn new(
        camera: &'a Camera<impl CameraProjection>,
        directional_lights: &'a [DirectionalLight],
        point_lights: &'a [PointLight],
    ) -> Self {
        assert!(
            directional_lights.len() <= MAX_DIRECTIONAL_LIGHTS,
            "A render context supports at most {MAX_DIRECTIONAL_LIGHTS} directional lights, got {}.",
            directional_lights.len(),
        );
        assert!(
            point_lights.len() <= MAX_POINT_LIGHTS,
            "A render context supports at most {MAX_POINT_LIGHTS} point lights, got {}.",
            point_lights.len(),
        );
        Self {
            camera_uniforms: Box::new(move |aspect_ratio| {
                let mut uniforms = CameraUniforms::default();
                uniforms.update(camera, aspect_ratio);
                uniforms
            }),
            exposure: camera.exposure,
            directional_lights,
            point_lights,
        }
    }

    pub(crate) fn camera_uniforms(&self, aspect_ratio: f64) -> CameraUniforms {
        (self.camera_uniforms)(aspect_ratio)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::camera::{Orthographic, Perspective};
    use crate::photometry::Candela;
    use crate::transform::Transformable;
    use glam::{DMat4, DVec3};

    #[test]
    fn borrows_the_app_lights() {
        let camera: Camera = Camera::default();
        let directional_lights = [DirectionalLight::default(), DirectionalLight::default()];
        let mut point_lights = [PointLight::default()];
        point_lights[0].set_intensity(Candela::new(0.2));

        let context = FrameBindingsData::new(&camera, &directional_lights, &point_lights);
        assert!(std::ptr::eq(
            context.directional_lights.as_ptr(),
            directional_lights.as_ptr()
        ));
        assert_eq!(context.directional_lights.len(), 2);
        assert!(std::ptr::eq(
            context.point_lights.as_ptr(),
            point_lights.as_ptr()
        ));
        assert_eq!(context.point_lights[0].intensity(), Candela::new(0.2));
    }

    #[test]
    fn resolves_camera_for_each_render_aspect_ratio() {
        let camera: Camera<Perspective> = Camera::default();
        camera.transform_mut().position = DVec3::new(3.0, 4.0, 5.0);
        let light = DirectionalLight::default();
        let context = FrameBindingsData::new(&camera, std::slice::from_ref(&light), &[]);

        for aspect_ratio in [1.0, 2.0, 0.5] {
            let resolved: [f32; 20] = bytemuck::cast(context.camera_uniforms(aspect_ratio));
            assert_eq!(
                resolved[..16],
                camera
                    .projection_view(aspect_ratio)
                    .as_mat4()
                    .to_cols_array()
            );
            assert_eq!(resolved[16..19], [3.0, 4.0, 5.0]);
        }
    }

    #[test]
    fn different_projection_types_produce_the_same_bindings_type() {
        struct CustomProjection(Perspective);
        impl CameraProjection for CustomProjection {
            fn matrix(&self, aspect_ratio: f64) -> DMat4 {
                self.0.matrix(aspect_ratio)
            }
        }

        let perspective: Camera<Perspective> = Camera::default();
        let orthographic = Camera::new(Orthographic {
            size_vertical: 4.0,
            near: 0.1,
            far: 100.0,
        });
        let custom = Camera::new(CustomProjection(Perspective {
            fov_vertical: 1.0,
            ..Default::default()
        }));
        let light = DirectionalLight::default();
        let frames = [
            FrameBindingsData::new(&perspective, std::slice::from_ref(&light), &[]),
            FrameBindingsData::new(&orthographic, std::slice::from_ref(&light), &[]),
            FrameBindingsData::new(&custom, std::slice::from_ref(&light), &[]),
        ];
        let matrices = [
            perspective.projection_view(1.5),
            orthographic.projection_view(1.5),
            custom.projection_view(1.5),
        ];
        for (frame, matrix) in frames.into_iter().zip(matrices) {
            let camera: [f32; 20] = bytemuck::cast(frame.camera_uniforms(1.5));
            assert_eq!(camera[..16], matrix.as_mat4().to_cols_array());
        }
    }

    #[test]
    fn accepts_no_lights() {
        let camera: Camera = Camera::default();
        let context = FrameBindingsData::new(&camera, &[], &[]);
        assert!(context.directional_lights.is_empty());
        assert!(context.point_lights.is_empty());
    }

    #[test]
    fn accepts_the_maximum_directional_light_count() {
        let camera: Camera = Camera::default();
        let lights =
            std::array::from_fn::<_, MAX_DIRECTIONAL_LIGHTS, _>(|_| DirectionalLight::default());
        let context = FrameBindingsData::new(&camera, &lights, &[]);
        assert_eq!(context.directional_lights.len(), MAX_DIRECTIONAL_LIGHTS);
    }

    #[test]
    #[should_panic(expected = "directional lights")]
    fn rejects_more_directional_lights_than_material_buffers_support() {
        let camera: Camera = Camera::default();
        let lights = std::array::from_fn::<_, { MAX_DIRECTIONAL_LIGHTS + 1 }, _>(|_| {
            DirectionalLight::default()
        });
        FrameBindingsData::new(&camera, &lights, &[]);
    }

    #[test]
    fn accepts_no_point_lights() {
        let camera: Camera = Camera::default();
        let light = DirectionalLight::default();
        let context = FrameBindingsData::new(&camera, std::slice::from_ref(&light), &[]);
        assert!(context.point_lights.is_empty());
    }

    #[test]
    fn accepts_the_maximum_point_light_count() {
        let camera: Camera = Camera::default();
        let light = DirectionalLight::default();
        let point_lights = std::array::from_fn::<_, MAX_POINT_LIGHTS, _>(|_| PointLight::default());
        let context = FrameBindingsData::new(&camera, std::slice::from_ref(&light), &point_lights);
        assert_eq!(context.point_lights.len(), MAX_POINT_LIGHTS);
    }

    #[test]
    #[should_panic(expected = "A render context supports at most")]
    fn rejects_more_point_lights_than_material_buffers_support() {
        let camera: Camera = Camera::default();
        let light = DirectionalLight::default();
        let point_lights =
            std::array::from_fn::<_, { MAX_POINT_LIGHTS + 1 }, _>(|_| PointLight::default());
        FrameBindingsData::new(&camera, std::slice::from_ref(&light), &point_lights);
    }
}
