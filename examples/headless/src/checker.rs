use gilgamesh::materials::{Material, RasterMaterial};

/// An app-defined emissive checkerboard; RGB values use the scene luminance scale.
#[derive(Clone)]
pub(super) struct CheckerMaterial {
    pub colors: [[f32; 3]; 2],
    pub scale: f32,
}

impl Material for CheckerMaterial {
    fn name(&self) -> &'static str {
        "Checkerboard"
    }

    fn raster(&self) -> RasterMaterial<'_> {
        let [a, b] = self.colors;
        RasterMaterial::new(
            include_str!("checker.wgsl"),
            [
                [a[0], a[1], a[2], 1.0],
                [b[0], b[1], b[2], 1.0],
                [self.scale, self.scale, 0.0, 0.0],
            ],
        )
    }
}
