#![allow(dead_code)]

pub mod camera;
pub mod texture;
pub mod model;
pub mod light;
pub mod render_target;
pub mod text;
pub mod texture_atlas;

pub mod prelude {
    pub use crate::{
        *,
        camera::*,
        texture::*,
        model::*,
        light::*,
        render_target::*,
        text::*,
        texture_atlas::*,
    };
}
pub use prelude::*;

/************************************************
 * General structs
 ************************************************/
pub trait Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a>;
}


#[derive(Debug, Clone, Copy)]
pub struct Color {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}
impl Color {
    pub fn from_rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self{ r, g, b, a}
    }
    pub fn from_rgb(r: f32, g: f32, b: f32) -> Self {
        Color::from_rgba(r,g,b,1.0)
    }
    pub fn as_array(&self) -> [f32;4] {
        [self.r, self.g, self.b, self.a]
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Position {
    x: f32,
    y: f32,
    z: f32,
}
impl Position {
    pub fn as_array(&self) -> [f32;3] {
        [self.x, self.y, self.z]
    }
}

/// Stores a viewport in screenspace coordinates (-1.0, 1.0)
#[derive(Debug, Clone, Copy)]
pub struct Viewport {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub min_depth: f32,
    pub max_depth: f32,
}

impl Viewport {
    pub fn new(x:f32, y:f32, width:f32, height:f32) -> Self {
        Viewport {
            x,
            y,
            width,
            height,
            min_depth: 0.0,
            max_depth: 1.0,
        }
    }
}

/************************************************
 * Tests
 ************************************************/

#[test]
fn test_empty_frame() {
    use winit::platform::windows::EventLoopExtWindows;
    let event_loop = winit::event_loop::EventLoop::<u8>::new_any_thread();
    let window = winit::window::WindowBuilder::new().build(&event_loop).unwrap();
    let mut render_target = futures::executor::block_on(render_target::RenderTarget::new(&window, false, None));
    render_target.begin_frame();
    render_target.end_frame();
}
