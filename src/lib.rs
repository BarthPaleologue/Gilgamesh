#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

pub mod animation;
pub mod camera;
pub mod core;
pub mod demo_scene_1;
pub mod demo_scene_2;
pub mod geometry;
pub mod input;
pub mod lights;
pub mod materials;
pub mod settings;
pub mod texture;
pub mod transform;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn run() {
    demo_scene_1::run();
}
