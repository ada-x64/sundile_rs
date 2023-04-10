use wgpu::{BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource};

use crate::{texture::TextureWrapper, RenderTarget};
use std::collections::HashMap;

/// Sprite struct. Always contained within a TextureAtlas.
pub struct Sprite {
    pub frames: Vec<[u32; 2]>, //Top-left position [x,y] in pixel coordinates on the texture atlas texture.
    pub width: u32,
    pub height: u32,
    pub num_frames: f32,
    pub frame: f32,
    pub fps: f32,
}
impl Sprite {
    pub fn new(frames: Vec<[u32; 2]>, width: u32, height: u32, num_frames: u32, fps: f32) -> Self {
        Self {
            frames,
            num_frames: num_frames as f32,
            width,
            height,
            frame: 0.0,
            fps,
        }
    }
    pub fn update(&mut self, dt: f32) {
        if self.num_frames > 0.0 {
            self.frame += self.fps * dt;
            if self.frame > self.num_frames {
                self.frame -= self.num_frames;
            }
        }
    }
    pub fn current_frame(&self) -> [u32; 2] {
        self.frames[self.frame.floor() as usize]
    }
}

pub struct TextureAtlas {
    pub texture: TextureWrapper,
    pub bind_group: BindGroup,
    pub spritemap: HashMap<String, Sprite>,
}
impl TextureAtlas {
    pub fn new(
        render_target: &RenderTarget,
        layout: &wgpu::BindGroupLayout,
        texture: TextureWrapper,
        spritemap: HashMap<String, Sprite>,
    ) -> Self {
        // let texture = texture::Texture::load(&render_target.device, &render_target.queue, "assets/textures/atlas_0.png", false).expect("Unable to create texture atlas!");
        // let texture = texture::Texture::from_bytes(&render_target.device, &render_target.queue, bytes, "2D Texture Atlas", false).expect("Unable to create texture atlas!");

        let bind_group = render_target
            .device
            .create_bind_group(&BindGroupDescriptor {
                label: Some("2D Texture Atlas Bind Group"),
                layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(&texture.view),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::Sampler(&texture.sampler),
                    },
                ],
            });

        Self {
            texture,
            bind_group,
            spritemap,
        }
    }
}

/// Grid settings for [TextureAtlas] creation. Determines how a texture will be sliced into sprites. All units are in pixels.
pub struct SpriteSheet {
    pub sprite_width: u32,
    pub sprite_height: u32,
    pub separation_x: u32,
    pub separation_y: u32,
    pub offset_x: u32,
    pub offset_y: u32,
}
impl SpriteSheet {
    /// Creates a new SpriteSheet
    pub fn new(
        sprite_width: u32,
        sprite_height: u32,
        separation_x: u32,
        separation_y: u32,
        offset_x: u32,
        offset_y: u32,
    ) -> Self {
        Self {
            sprite_width,
            sprite_height,
            separation_x,
            separation_y,
            offset_x,
            offset_y,
        }
    }
    /// Creates a SpriteSheet using the full texture.
    pub fn full(texture: &TextureWrapper) -> Self {
        Self {
            sprite_width: texture.size.width,
            sprite_height: texture.size.height,
            separation_x: 0,
            separation_y: 0,
            offset_x: 0,
            offset_y: 0,
        }
    }
    /// Creates a SpriteSheet from relative coordinates in the range [0.0, 1.0].
    pub fn from_relative(
        texture: &TextureWrapper,
        sprite_width: f32,
        sprite_height: f32,
        separation_x: f32,
        separation_y: f32,
        offset_x: f32,
        offset_y: f32,
    ) -> Self {
        let tw = texture.size.width as f32;
        let th = texture.size.height as f32;
        Self {
            sprite_width: (tw * sprite_width) as u32,
            sprite_height: (th * sprite_height) as u32,
            separation_x: (tw * separation_x) as u32,
            separation_y: (th * separation_y) as u32,
            offset_x: (tw * offset_x) as u32,
            offset_y: (th * offset_y) as u32,
        }
    }
}

/// Builder for a [TextureAtlas]
pub struct TextureAtlasBuilder<'a> {
    map: HashMap<String, (&'a TextureWrapper, SpriteSheet)>,
}
impl<'a> TextureAtlasBuilder<'a> {
    /// Creates a new builder.
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
    /// Adds a texture to be split with a [SpriteSheet]. This will create a [Sprite] with multiple frames.
    pub fn with_sprite_sheet<S>(
        mut self,
        name: S,
        texture: &'a TextureWrapper,
        sprite_sheet: SpriteSheet,
    ) -> Self
    where
        S: Into<String>,
    {
        self.map.insert(name.into(), (texture, sprite_sheet));
        self
    }
    /// Adds a texture. This will create a [Sprite] with a single frame.
    pub fn with_texture<S>(mut self, name: S, texture: &'a TextureWrapper) -> Self
    where
        S: Into<String>,
    {
        let ss = SpriteSheet::full(&texture);
        self.map.insert(name.into(), (texture, ss));
        self
    }
    /// Combines all the sprite sheets into a single, compact texture.
    pub fn build() -> TextureAtlas {
        //TODO: Generate a compacted texture here and store it in a TextureAtlas.
        todo!()
    }
}

