use super::*;
use gilgamesh::materials::{Material, RasterMaterial};
use gilgamesh::photometry::Lux;

struct BlueMaterial;

impl Material for BlueMaterial {
    fn name(&self) -> &'static str {
        "Blue"
    }

    fn raster(&self) -> RasterMaterial<'_> {
        RasterMaterial::new(
            r"
            @group(1) @binding(0) var<uniform> luminance: vec4<f32>;
            @fragment
            fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
                return vec4(0.0, 0.0, luminance.x, 1.0);
            }
        ",
            [10_000.0_f32, 0.0, 0.0, 0.0],
        )
    }
}

#[test]
#[ignore = "GPU end-to-end check; run explicitly with approval"]
fn custom_materials_mix_with_pbr_and_can_be_replaced() {
    HeadlessRuntime::<HeadlessScene>::run(64, 64, async |mut runtime| {
        let mut checker = Rc::new(checker::CheckerMaterial {
            colors: [[10_000.0, 0.0, 0.0], [0.0, 10_000.0, 0.0]],
            scale: 8.0,
        });
        let mut sphere = PbrMaterial::new("Blue emitter");
        sphere.set_emissive_luminance(Nits::new(10_000.0));
        sphere.set_emissive_color(0.0, 0.0, 1.0);
        {
            let app = runtime.app_mut();
            app.camera.transform_mut().position = DVec3::new(0.0, 5.0, 0.0);
            app.camera
                .transform_mut()
                .look_at(DVec3::ZERO, DVec3::NEG_Z);
            app.directional_lights[0].set_illuminance(Lux::new(0.0));
            app.plane.set_material(checker.clone());
            app.sphere.set_material(Rc::new(sphere));
        }
        runtime.render(Duration::ZERO);
        let first = runtime.capture_frame().await.unwrap();
        assert!(first.pixels().any(|p| p[0] > 80 && p[1] == 0 && p[2] == 0));
        assert!(first.pixels().any(|p| p[1] > 80 && p[0] == 0 && p[2] == 0));
        let blue = first.get_pixel(32, 32);
        assert!(blue[2] > 80 && blue[0] == 0 && blue[1] == 0);

        Rc::make_mut(&mut checker).colors = [[10_000.0, 10_000.0, 0.0]; 2];
        runtime.app_mut().plane.set_material(checker);
        runtime.render(Duration::ZERO);
        let second = runtime.capture_frame().await.unwrap();
        assert!(second.pixels().any(|p| p[0] > 80 && p[1] > 80 && p[2] == 0));
        assert!(!second.pixels().any(|p| p[0] > 80 && p[1] == 0 && p[2] == 0));
        assert_eq!(second.get_pixel(32, 32), blue);
        runtime.render(Duration::ZERO);
        assert_eq!(runtime.capture_frame().await.unwrap(), second);

        runtime.app_mut().plane.set_material(Rc::new(BlueMaterial));
        runtime.render(Duration::ZERO);
        let third = runtime.capture_frame().await.unwrap();
        assert!(third.pixels().all(|p| p[0] == 0 && p[1] == 0 && p[2] > 80));
    });
}
