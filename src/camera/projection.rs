use glam::{DMat4, dcamera::rh::proj::directx};

pub trait CameraProjection {
    /// Maps camera space (-Z forward, +Y up) to WebGPU clip space with reverse depth:
    /// the near plane maps to 1 and the far plane to 0.
    fn matrix(&self) -> DMat4;
}

/// Perspective projection, with angles in radians and distances in world units.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Perspective {
    pub fov_vertical: f64,
    pub aspect_ratio: f64,
    pub near: f64,
    pub far: f64,
}

impl Default for Perspective {
    fn default() -> Self {
        Self {
            fov_vertical: 70.0_f64.to_radians(),
            aspect_ratio: 1.0,
            near: 0.1,
            far: 100.0,
        }
    }
}

impl CameraProjection for Perspective {
    fn matrix(&self) -> DMat4 {
        directx::perspective(self.fov_vertical, self.aspect_ratio, self.far, self.near)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Orthographic {
    /// Full visible height in world units, independent of distance from the camera.
    pub size_vertical: f64,
    pub aspect_ratio: f64,
    pub near: f64,
    pub far: f64,
}

impl CameraProjection for Orthographic {
    fn matrix(&self) -> DMat4 {
        let half_height = self.size_vertical / 2.0;
        let half_width = half_height * self.aspect_ratio;
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

/// Asymmetric perspective with signed angles from the camera's -Z axis.
/// Angles increase toward the right and top, and lie between -π/2 and π/2.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OffAxis {
    pub angle_left: f64,
    pub angle_right: f64,
    pub angle_bottom: f64,
    pub angle_top: f64,
    pub near: f64,
    pub far: f64,
}

impl CameraProjection for OffAxis {
    fn matrix(&self) -> DMat4 {
        // Bounds lie on the swapped near plane to preserve the angular field of view.
        directx::frustum(
            self.far * self.angle_left.tan(),
            self.far * self.angle_right.tan(),
            self.far * self.angle_bottom.tan(),
            self.far * self.angle_top.tan(),
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
    OffAxis(OffAxis),
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

impl From<OffAxis> for Projection {
    fn from(projection: OffAxis) -> Self {
        Self::OffAxis(projection)
    }
}

impl CameraProjection for Projection {
    fn matrix(&self) -> DMat4 {
        match self {
            Self::Perspective(projection) => projection.matrix(),
            Self::Orthographic(projection) => projection.matrix(),
            Self::OffAxis(projection) => projection.matrix(),
        }
    }
}
