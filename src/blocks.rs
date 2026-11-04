//! Built-in render graph blocks. Compose them with ordinary Rust functions.

mod rasterize;
mod resize_nearest;
mod tone_map;
mod vertical_split_screen;
mod visualize_depth;

pub use rasterize::{RasterOutput, rasterize};
pub use resize_nearest::resize_nearest;
pub use tone_map::tone_map;
pub use vertical_split_screen::vertical_split_screen;
pub use visualize_depth::visualize_depth;
