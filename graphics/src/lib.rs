pub mod camera;
pub mod light;
pub mod model;
pub mod render_target;
pub mod text;
pub mod texture;
pub mod texture_atlas;

pub mod prelude {
    pub use crate::{
        camera::*, light::*, model::*, render_target::*, text::*, texture::*, texture_atlas::*, *,
    };
    pub use image;
}
pub use prelude::*;

/************************************************
 * General structs
 ************************************************/
pub trait Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a>;
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vert2d {
    pub position: [f32; 3],
    pub color: [f32; 4],
    pub texcoords: [f32; 2],
}

impl Vertex for Vert2d {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vert2d>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 7]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
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
        Self { r, g, b, a }
    }
    pub fn from_rgb(r: f32, g: f32, b: f32) -> Self {
        Color::from_rgba(r, g, b, 1.0)
    }
    pub fn as_array(&self) -> [f32; 4] {
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
    pub fn as_array(&self) -> [f32; 3] {
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
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
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

#[cfg(target_arch = "windows")]
#[test]
fn test_empty_frame() {
    use winit::platform::windows::EventLoopExtWindows;
    let event_loop = winit::event_loop::EventLoop::<u8>::new_any_thread();
    let window = winit::window::WindowBuilder::new()
        .build(&event_loop)
        .unwrap();
    let mut render_target =
        futures::executor::block_on(render_target::RenderTarget::new(&window, false, None));
    render_target.begin_frame();
    render_target.end_frame();
}
