use crate::core::render_context::RenderContext;
use crate::core::wgpu_context::WGPUContext;
use crate::geometry::mesh::Mesh;
use crate::input::mouse::Mouse;
use winit::event::WindowEvent;

pub struct FrameContext<'a> {
    pub elapsed_seconds: f32,
    pub delta_seconds: f32,
    pub mouse: &'a Mouse,
    pub wgpu_context: &'a WGPUContext,
}

pub trait App {
    /// Updates application state before the engine borrows its render context.
    fn update_before_render(&mut self, frame: &FrameContext<'_>);

    /// Borrows the current objects, camera, and lights for rendering or camera resizing.
    fn get_render_context(&mut self) -> RenderContext<'_, impl IntoIterator<Item = &mut Mesh>>;

    fn on_window_event(&mut self, _event: &WindowEvent) {}
}
