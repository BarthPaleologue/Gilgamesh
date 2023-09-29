#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

pub mod camera;
pub mod transform;
pub mod material;
pub mod geometry;
pub mod core;
pub mod input;