//! Built-in render graph blocks. Compose them with ordinary Rust functions.

mod rasterize;
mod tone_map;

pub use rasterize::{RasterOutput, rasterize};
pub use tone_map::tone_map;
