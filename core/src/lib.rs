pub mod defaults;
pub mod game;
pub mod renderer;
pub mod renderer2d;
pub mod scene;

pub mod prelude {
    pub use crate::game::*;
    pub use crate::renderer::*;
    pub use crate::renderer2d::*;
    pub use crate::scene::*;
}
pub use prelude::*;
