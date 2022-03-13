mod tests;
pub mod components;
pub mod scene;

pub mod prelude {
    pub use crate::components::*;
    pub use crate::scene::*;
}

// Global API struct that acts as as bridge between API functions and Game struct.
// /// Maybe move debug_gui here?
// struct API {

// }

// pub fn initialize_api() {
//     //???
// }