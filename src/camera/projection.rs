use glam::{DMat4, DVec2, dcamera::rh::proj::directx};

pub trait CameraProjection {
    /// Maps camera space (-Z forward, +Y up) to WebGPU clip space with reverse depth:
    /// the near plane maps to 1; depth reaches 0 at the far plane or approaches it at infinity.
    /// `aspect_ratio` is the positive, finite width/height ratio of the rendered view.
    fn matrix(&self, aspect_ratio: f64) -> DMat4;
}

/// Perspective projection with an infinite far plane.
/// Angles are in radians and distances in world units.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Perspective {
    /// Vertical opening before applying the frustum offset, in radians.
    pub fov_vertical: f64,
    /// Frustum center in units of half-width and half-height; positive is right/up.
    /// Zero centers the projection. Values outside [-1, 1] are valid.
    pub frustum_offset: DVec2,
    pub near: f64,
}

impl Default for Perspective {
    fn default() -> Self {
        Self {
            fov_vertical: 70.0_f64.to_radians(),
            frustum_offset: DVec2::ZERO,
            near: 0.1,
        }
    }
}

impl CameraProjection for Perspective {
    fn matrix(&self, aspect_ratio: f64) -> DMat4 {
        let mut matrix =
            directx::perspective_infinite_reverse(self.fov_vertical, aspect_ratio, self.near);
        matrix.z_axis.x = self.frustum_offset.x;
        matrix.z_axis.y = self.frustum_offset.y;
        matrix
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Orthographic {
    /// Full visible height in world units, independent of distance from the camera.
    pub size_vertical: f64,
    pub near: f64,
    pub far: f64,
}

impl CameraProjection for Orthographic {
    fn matrix(&self, aspect_ratio: f64) -> DMat4 {
        let half_height = self.size_vertical / 2.0;
        let half_width = half_height * aspect_ratio;
        directx::orthographic(
            -half_width,
            half_width,
            -half_height,
            half_height,
            self.far,
            self.near,
        )
    }
}

/// Projection selected at runtime, for applications that switch projection types.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Projection {
    Perspective(Perspective),
    Orthographic(Orthographic),
}

impl Default for Projection {
    fn default() -> Self {
        Self::Perspective(Perspective::default())
    }
}

impl From<Perspective> for Projection {
    fn from(projection: Perspective) -> Self {
        Self::Perspective(projection)
    }
}

impl From<Orthographic> for Projection {
    fn from(projection: Orthographic) -> Self {
        Self::Orthographic(projection)
    }
}

impl CameraProjection for Projection {
    fn matrix(&self, aspect_ratio: f64) -> DMat4 {
        match self {
            Self::Perspective(projection) => projection.matrix(aspect_ratio),
            Self::Orthographic(projection) => projection.matrix(aspect_ratio),
        }
    }
}
