///
/// 2d.rs
///
/// Contains API for in-game GUI / 2d overlay drawing.
///
// Needed functionality:
// Draw quad w/ color & alpha
// Draw textured quad
// Draw text
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use sundile_assets::AssetTypeMap;
use sundile_graphics::{
    Color, Font, FontSpecifier, GlyphRenderer, RenderTarget, Sprite, TextBlock, TextureAtlas,
    Vert2d, Vertex,
};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::*;
use wgpu_glyph::{
    BuiltInLineBreaker, FontId, HorizontalAlign, Layout, Section, Text, VerticalAlign,
};

struct Quad {
    sprite: Option<&'static str>,
    vertices: [f32; 4],
    color: Color,
}

const DEFAULT_LAYOUT: Layout<BuiltInLineBreaker> = Layout::SingleLine {
    line_breaker: BuiltInLineBreaker::UnicodeLineBreaker,
    h_align: HorizontalAlign::Left,
    v_align: VerticalAlign::Bottom,
};
const DEFAULT_FONT: FontId = FontId(0);

/// This struct is a wrapper for wgpu_glyph's Section class.
/// It contains all the parameters needed to create the section,
/// which will be instantiated when needed.
/// This avoids lifetime shenanegins.
struct SectionBuilder {
    text: String,
    pos: (f32, f32),
    bounds: (f32, f32),
    color: Color,
    font_size: f32,
    current_font: FontId,
    layout: Layout<BuiltInLineBreaker>,
}
impl SectionBuilder {
    /// Build the Section.
    /// Note: Non-consuming due to lifetime constraints.
    pub fn build<'a>(&'a self) -> Section<'a> {
        Section {
            screen_position: self.pos,
            bounds: self.bounds,
            text: vec![Text::new(self.text.as_str())
                .with_color(self.color.as_array())
                .with_scale(self.font_size)
                .with_font_id(self.current_font)],
            layout: self.layout,
            ..Section::default()
        }
    }
}

/// The 2D Renderer is designed for GUI elements. Anything that will be drawn
/// directly to the screen. Useful for HUDs and debug info.
/// TODO: This seems unnecessary. Just create a quad that is always drawn
/// right next to the camera.
pub struct Renderer2d {
    texture_atlas: TextureAtlas,
    queue: Vec<Quad>,
    pipeline: wgpu::RenderPipeline,
    color: Color,
    screen_size: [u32; 2],

    text_wrapper: GlyphRenderer,
    text_queue: Vec<SectionBuilder>,
    text_bounds: (f32, f32),
    font_size: f32,
    current_font: Option<FontId>,
    current_layout: Option<Layout<BuiltInLineBreaker>>,
}

#[allow(dead_code)]
impl Renderer2d {
    pub fn new(render_target: &RenderTarget, assets: &mut AssetTypeMap) -> Self {
        let (device, config) = (&render_target.device, &render_target.config);

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Renderer2D Texture Bind Group Layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            multisampled: false,
                            view_dimension: TextureViewDimension::D2,
                            sample_type: TextureSampleType::Float { filterable: false },
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::NonFiltering),
                        count: None,
                    },
                ],
            });
        let layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Renderer2D Pipeline Layout Descriptor"),
            bind_group_layouts: &[&texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        let shader = assets.try_get_asset::<wgpu::ShaderModule>("2d").unwrap();

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Renderer2D Pipeline"),
            layout: Some(&layout),
            vertex: VertexState {
                module: shader.as_ref(),
                entry_point: "vs_main",
                buffers: &[Vert2d::desc()],
            },
            fragment: Some(FragmentState {
                module: shader.as_ref(),
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: config.format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        let text_wrapper =
            GlyphRenderer::new(&render_target, assets.try_take_asset_map::<Font>().ok());

        // let texture_atlas = TextureAtlasBuilder::new()
        //     .with_sprite_sheet("atlas_0", assets.get_asset("textures", "atlas_0"), SpriteSheet::new(16,16,0,0,0,0))
        //     .build();

        //TODO: Actually load in texture atlases!!!
        let texture_atlas = TextureAtlas::new(
            render_target,
            &texture_bind_group_layout,
            assets.try_take_asset("test_atlas").unwrap(),
            HashMap::from_iter([
                ("default".into(), Sprite::new(vec![[0, 0]], 16, 16, 1, 0.0)),
                ("circle".into(), Sprite::new(vec![[16, 0]], 16, 16, 1, 0.0)),
            ]),
        );

        Self {
            texture_atlas,
            queue: vec![],
            pipeline,
            color: Color::from_rgb(1.0, 1.0, 1.0),
            screen_size: [render_target.config.width, render_target.config.height],

            text_wrapper,
            text_queue: vec![],
            text_bounds: (
                render_target.config.width as f32,
                render_target.config.height as f32,
            ),
            font_size: 16.0,
            current_font: None,
            current_layout: None,
        }
    }

    pub fn render(&mut self, render_target: &mut RenderTarget, assets: Arc<Mutex<AssetTypeMap>>) {
        // Update any text assets.
        let lock = assets.lock();
        let assets = lock.unwrap();
        if let Ok(textblocks) = assets.try_get_asset_map::<TextBlock>() {
            textblocks.iter().for_each(|(_, value)| {
                value.instance_cache.iter().for_each(|instance| {
                    if let Some(font) = instance.font.as_ref() {
                        self.set_font(font);
                    }
                    if let Some(layout) = instance.layout.as_ref() {
                        self.set_text_layout(layout);
                    }
                    if instance.relative_position {
                        self.draw_text_rel(value.data.clone(), instance.x, instance.y)
                    } else {
                        self.draw_text(value.data.clone(), instance.x, instance.y)
                    }
                })
            })
        }
        drop(assets);

        // smoosh quads into batch
        let mut vertices: Vec<Vert2d> = vec![];
        let mut indices: Vec<u32> = vec![];

        while let Some(quad) = self.queue.pop() {
            let sprite = &self.texture_atlas.spritemap[quad.sprite.unwrap_or("default")];

            let tw = self.texture_atlas.texture.size.width as f32;
            let th = self.texture_atlas.texture.size.height as f32;
            let (x1, y1, x2, y2) = (
                quad.vertices[0],
                quad.vertices[1],
                quad.vertices[0] + quad.vertices[2],
                quad.vertices[1] + quad.vertices[3],
            );
            let sxy = sprite.current_frame();
            let (tx1, ty1, tx2, ty2) = (
                sxy[0] as f32 / tw,
                sxy[1] as f32 / th,
                (sxy[0] + sprite.width) as f32 / tw,
                (sxy[1] + sprite.height) as f32 / th,
            );

            vertices.push(Vert2d {
                position: [x1, y1, 0.0],
                texcoords: [tx1, ty1],
                color: quad.color.as_array(),
            });
            vertices.push(Vert2d {
                position: [x1, y2, 0.0],
                texcoords: [tx1, ty2],
                color: quad.color.as_array(),
            });
            vertices.push(Vert2d {
                position: [x2, y1, 0.0],
                texcoords: [tx2, ty1],
                color: quad.color.as_array(),
            });
            vertices.push(Vert2d {
                position: [x2, y2, 0.0],
                texcoords: [tx2, ty2],
                color: quad.color.as_array(),
            });

            let i0 = (vertices.len() - 4) as u32;
            indices.push(i0);
            indices.push(i0 + 1);
            indices.push(i0 + 2);
            indices.push(i0 + 1);
            indices.push(i0 + 3);
            indices.push(i0 + 2);
        }

        let device = &render_target.device;
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("2D Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("2D Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        // render batch
        let mut rp = render_target.get_render_pass(false, false);
        rp.set_pipeline(&self.pipeline);
        rp.set_vertex_buffer(0, vertex_buffer.slice(..));
        rp.set_index_buffer(index_buffer.slice(..), IndexFormat::Uint32);
        rp.set_bind_group(0, &self.texture_atlas.bind_group, &[]);
        rp.draw_indexed(0..indices.len() as u32, 0, 0..1);
        drop(rp);

        // render text
        self.text_wrapper.start_pass();
        while let Some(section) = self.text_queue.pop() {
            self.text_wrapper.queue_section(section.build());
        }
        self.text_wrapper.end_pass(render_target);
    }

    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    /// Draws at the given screen coordinates in the range (-1.0, 1.0).
    fn push_quad(&mut self, x: f32, y: f32, width: f32, height: f32, sprite: Option<&'static str>) {
        self.queue.push(Quad {
            vertices: [x, y, width, height],
            sprite,
            color: self.color,
        });
    }

    /// Draw quad at relative coordinates, in range (0.0, 1.0)
    pub fn draw_quad_rel(&mut self, x: f32, y: f32, width: f32, height: f32) {
        self.push_quad(
            x * 2.0 - 1.0,
            y * 2.0 - 1.0,
            width * 2.0,
            height * 2.0,
            None,
        );
    }

    /// Draw quad at pixel coordinates, with top-left at (0,0)
    pub fn draw_quad(&mut self, x: f32, y: f32, width: f32, height: f32) {
        let sw = self.screen_size[0] as f32;
        let sh = self.screen_size[1] as f32;
        self.push_quad(
            (x / sw) * 2.0 - 1.0,
            (y / sh) * 2.0 - 1.0,
            (width / sw) * 2.0,
            (height / sh) * 2.0,
            None,
        )
    }

    /// Draw a sprite at the given pixel coordinates, with top-left at (0,0).
    pub fn draw_sprite(
        &mut self,
        x: f32,
        y: f32,
        width_multiplier: f32,
        height_multiplier: f32,
        sprite: &'static str,
    ) {
        let spr = &self.texture_atlas.spritemap[sprite]; //TODO: Don't borrow the sprite here for performance reasons?
        let spr_width = spr.width as f32;
        let spr_height = spr.height as f32;
        let sw = self.screen_size[0] as f32;
        let sh = self.screen_size[1] as f32;
        self.push_quad(
            (x / sw) * 2.0 - 1.0,
            (y / sh) * 2.0 - 1.0,
            (spr_width * width_multiplier / sw) * 2.0,
            (spr_height * height_multiplier / sh) * 2.0,
            Some(sprite),
        );
    }

    /// Sets bounding box for text.
    pub fn set_text_bounds(&mut self, width: f32, height: f32) {
        self.text_bounds = (width, height);
    }

    /// Sets text layout.
    /// Seet wgpu_glyph::Layout
    pub fn set_text_layout(&mut self, layout: &Layout<BuiltInLineBreaker>) {
        self.current_layout = Some(layout.clone());
    }
    /// Sets current font.
    pub fn set_font(&mut self, font: &FontSpecifier) {
        self.current_font = Some(self.text_wrapper.font(&font.name));
        self.font_size = font.size;
    }

    /// Draws text at the given pixel coordinates.
    pub fn draw_text(&mut self, text: String, x: f32, y: f32) {
        self.text_queue.push(SectionBuilder {
            text,
            pos: (x, y),
            bounds: self.text_bounds,
            color: self.color,
            font_size: self.font_size,
            current_font: self.current_font.unwrap_or(DEFAULT_FONT),
            layout: self.current_layout.unwrap_or(DEFAULT_LAYOUT),
        });
    }

    pub fn draw_text_rel(&mut self, text: String, x: f32, y: f32) {
        self.draw_text(
            text,
            x * self.screen_size[0] as f32,
            y * self.screen_size[1] as f32,
        );
    }
}
