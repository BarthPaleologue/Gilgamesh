use crate::camera::{Camera, CameraProjection, Projection};
use crate::lights::directional_light::DirectionalLight;
use crate::lights::point_light::PointLight;
use crate::settings::MAX_POINT_LIGHTS;

/// Borrows the application's objects for a render pass without taking ownership.
pub struct FrameBindingsData<'a, ProjectionType: CameraProjection = Projection> {
    pub camera: &'a Camera<ProjectionType>,
    pub(crate) directional_light: &'a DirectionalLight,
    pub(crate) point_lights: &'a [PointLight],
}

impl<'a, ProjectionType: CameraProjection> FrameBindingsData<'a, ProjectionType> {
    pub fn new(
        camera: &'a Camera<ProjectionType>,
        directional_light: &'a DirectionalLight,
        point_lights: &'a [PointLight],
    ) -> Self {
        assert!(
            point_lights.len() <= MAX_POINT_LIGHTS,
            "A render context supports at most {MAX_POINT_LIGHTS} point lights, got {}.",
            point_lights.len(),
        );
        Self {
            camera,
            directional_light,
            point_lights,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lights::light::Light;

    #[test]
    fn borrows_the_app_camera_and_lights() {
        let camera: Camera = Camera::default();
        let directional_light = DirectionalLight::default();
        let mut point_lights = [PointLight::default()];
        point_lights[0].set_intensity(0.2);

        let context = FrameBindingsData::new(&camera, &directional_light, &point_lights);
        assert!(std::ptr::eq(context.camera, &camera));
        assert!(std::ptr::eq(context.directional_light, &directional_light));
        assert!(std::ptr::eq(
            context.point_lights.as_ptr(),
            point_lights.as_ptr()
        ));
        assert_eq!(context.point_lights[0].intensity(), 0.2);
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
