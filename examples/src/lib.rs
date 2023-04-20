use wasm_bindgen::prelude::wasm_bindgen;
use winit::{dpi::PhysicalSize, window::WindowBuilder};

#[cfg(feature = "basic")]
pub mod basic;
#[cfg(feature = "model_import")]
pub mod model_import;
#[cfg(feature = "text")]
pub mod text;

#[wasm_bindgen(start)]
pub fn wasm_main() {
    // 480p
    // 4x3 ratio for web platform
    doit(640, 480);
}

// Create the main function for WASM target.
#[wasm_bindgen]
pub fn doit(width: u32, height: u32) {
    let window_builder = WindowBuilder::new().with_inner_size(PhysicalSize::new(width, height));

    #[cfg(feature = "basic")]
    basic::doit(window_builder);

    #[cfg(feature = "model_import")]
    model_import::doit(window_builder);

    #[cfg(feature = "text")]
    text::doit(window_builder);
}
