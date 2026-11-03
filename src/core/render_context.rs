use crate::camera::Camera;
use crate::geometry::mesh::Mesh;
use crate::lights::directional_light::DirectionalLight;
use crate::lights::point_light::PointLight;
use crate::settings::MAX_POINT_LIGHTS;

/// Borrows the application's objects for a render pass without taking ownership.
pub struct RenderContext<'a, TRenderObjects> {
    pub camera: &'a mut Camera,
    pub(crate) render_objects: TRenderObjects,
    pub(crate) directional_light: &'a DirectionalLight,
    pub(crate) point_lights: &'a [PointLight],
}

impl<'a, TRenderObjects> RenderContext<'a, TRenderObjects>
where
    TRenderObjects: IntoIterator<Item = &'a mut Mesh>,
{
    pub fn new(
        camera: &'a mut Camera,
        render_objects: TRenderObjects,
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
            render_objects,
            directional_light,
            point_lights,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lights::light::Light;
    use std::iter;

    #[test]
    fn borrows_the_app_camera_and_lights() {
        let mut camera = Camera::new(1.0);
        let directional_light = DirectionalLight::default();
        let mut point_lights = [PointLight::default()];
        point_lights[0].set_intensity(0.2);

        let context = RenderContext::new(
            &mut camera,
            iter::empty(),
            &directional_light,
            &point_lights,
        );
        context.camera.aspect_ratio = 2.0;
        assert!(std::ptr::eq(context.directional_light, &directional_light));
        assert!(std::ptr::eq(
            context.point_lights.as_ptr(),
            point_lights.as_ptr()
        ));
        assert_eq!(context.point_lights[0].intensity(), 0.2);
        assert_eq!(camera.aspect_ratio, 2.0);
    }

    #[test]
    fn accepts_no_objects_and_no_point_lights() {
        let mut camera = Camera::new(1.0);
        let light = DirectionalLight::default();
        let context = RenderContext::new(&mut camera, [], &light, &[]);
        assert_eq!(context.render_objects.len(), 0);
        assert!(context.point_lights.is_empty());
    }

    #[test]
    fn accepts_the_maximum_point_light_count() {
        let mut camera = Camera::new(1.0);
        let light = DirectionalLight::default();
        let point_lights = std::array::from_fn::<_, MAX_POINT_LIGHTS, _>(|_| PointLight::default());
        let context = RenderContext::new(&mut camera, [], &light, &point_lights);
        assert_eq!(context.point_lights.len(), MAX_POINT_LIGHTS);
    }

    #[test]
    #[should_panic(expected = "A render context supports at most")]
    fn rejects_more_point_lights_than_material_buffers_support() {
        let mut camera = Camera::new(1.0);
        let light = DirectionalLight::default();
        let point_lights =
            std::array::from_fn::<_, { MAX_POINT_LIGHTS + 1 }, _>(|_| PointLight::default());
        RenderContext::new(&mut camera, [], &light, &point_lights);
    }
}
