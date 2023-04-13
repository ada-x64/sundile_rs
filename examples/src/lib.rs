use wasm_bindgen::prelude::wasm_bindgen;

#[cfg(feature = "basic")]
pub mod basic;
#[cfg(feature = "model_import")]
pub mod model_import;
#[cfg(feature = "text")]
pub mod text;

// Create the main function for WASM target.
#[wasm_bindgen(start)]
pub fn doit() {
    #[cfg(feature = "basic")]
    basic::doit();

    #[cfg(feature = "model_import")]
    model_import::doit();

    #[cfg(feature = "text")]
    text::doit();
}
