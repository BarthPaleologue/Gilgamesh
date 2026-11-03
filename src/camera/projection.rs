use glam::{DMat4, dcamera::rh::proj::directx};

pub trait CameraProjection {
    /// Maps camera space (-Z forward, +Y up) to WebGPU clip space (depth 0..1).
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
        directx::perspective(self.fov_vertical, self.aspect_ratio, self.near, self.far)
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
            self.near,
            self.far,
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
        directx::frustum(
            self.near * self.angle_left.tan(),
            self.near * self.angle_right.tan(),
            self.near * self.angle_bottom.tan(),
            self.near * self.angle_top.tan(),
            self.near,
            self.far,
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

impl CameraProjection for Projection {
    fn matrix(&self) -> DMat4 {
        match self {
            Self::Perspective(projection) => projection.matrix(),
            Self::Orthographic(projection) => projection.matrix(),
            Self::OffAxis(projection) => projection.matrix(),
        }
    }
}
