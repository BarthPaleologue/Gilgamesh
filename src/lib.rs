#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[macro_use]
extern crate load_file;

pub mod camera;
pub mod transform;
pub mod geometry;
pub mod core;
pub mod input;
pub mod lights;
pub mod settings;
pub mod materials;
pub mod animation;