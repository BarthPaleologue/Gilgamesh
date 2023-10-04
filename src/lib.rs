#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[macro_use]
extern crate load_file;

extern crate image;

pub mod camera;
pub mod transform;
pub mod geometry;
pub mod core;
pub mod input;
pub mod lights;
pub mod settings;
pub mod materials;
pub mod animation;
pub mod texture;
pub mod demo_scene_1;
pub mod demo_scene_2;