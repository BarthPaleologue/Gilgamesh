use crate::camera::CameraProjection;
use crate::core::render_context::FrameBindingsData;
use crate::core::wgpu_context::WGPUContext;
use crate::geometry::mesh::Mesh;
use crate::input::mouse::Mouse;
use winit::event::WindowEvent;

pub struct FrameContext<'a> {
    pub elapsed_seconds: f64,
    pub delta_seconds: f64,
    /// Configured render surface width in physical pixels.
    pub surface_width: u32,
    /// Configured render surface height in physical pixels.
    pub surface_height: u32,
    pub mouse: &'a Mouse,
    pub wgpu_context: &'a WGPUContext,
}

pub trait App: Sized {
    /// Creates the application after WebGPU has been initialized.
    fn new(ctx: &WGPUContext) -> Self;

    /// Updates application state before the engine borrows its render context.
    fn update_before_render(&mut self, frame: &FrameContext<'_>);

    /// Borrows the camera and lights for rendering.
    fn get_frame_bindings_data(&mut self) -> FrameBindingsData<'_, impl CameraProjection>;

    /// Borrows the meshes.
    fn get_meshes(&mut self) -> impl IntoIterator<Item = &mut Mesh>;

    fn on_window_event(&mut self, _event: &WindowEvent) {}
}
