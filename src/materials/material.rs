use super::RasterMaterial;

/// Shared surface description. Renderers own the prepared GPU resources.
/// Configure before sharing, or replace the `Rc` when editing. Interior mutations
/// are not tracked by renderer caches.
pub trait Material {
    fn name(&self) -> &str;
    fn raster(&self) -> RasterMaterial<'_>;
}
