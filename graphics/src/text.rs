use crate::*;
use serde::*;
use std::collections::HashMap;
use wgpu::util::StagingBelt;
use wgpu_glyph::*;

/// Thin wrapper around a Vec<u8> of font data.
#[derive(Serialize, Deserialize)]
pub struct Font {
    pub data: Vec<u8>,
}

pub struct TextWrapper {
    staging_belt: StagingBelt,
    brush: GlyphBrush<()>,
    fonts: HashMap<String, FontId>,
    submitted_already: bool,
}

impl TextWrapper {
    pub fn new(render_target: &RenderTarget, raw_fonts: Option<HashMap<String, Font>>) -> Self {
        let staging_belt = StagingBelt::new(1024);

        let mut fonts = HashMap::<String, FontId>::new();
        let mut font_data = Vec::<ab_glyph::FontArc>::new();
        match raw_fonts {
            Some(raw_fonts) => {
                for (name, font) in raw_fonts {
                    fonts.insert(name, FontId(font_data.len()));
                    font_data.push(
                        ab_glyph::FontArc::try_from_vec(font.data.clone())
                            .expect("Unable to register font!"),
                    );
                }
            }
            None => {
                use log::warn;
                warn!("No fonts found!");
            }
        }
        let brush = GlyphBrushBuilder::using_fonts(font_data)
            .build(&render_target.device, render_target.config.format);

        Self {
            staging_belt,
            brush,
            fonts,
            submitted_already: false,
        }
    }

    pub fn start_pass(&mut self) {
        if self.submitted_already {
            self.staging_belt.recall();
            self.submitted_already = false;
        }
    }

    pub fn end_pass(&mut self, render_target: &mut RenderTarget) {
        self.brush
            .draw_queued(
                &render_target.device,
                &mut self.staging_belt,
                render_target.encoder.as_mut().unwrap(),
                render_target.color_view.as_ref().unwrap(),
                render_target.config.width,
                render_target.config.height,
            )
            .expect("Could not submit GlyphBrush queue!");
        self.staging_belt.finish();
        self.submitted_already = true;
    }

    pub fn queue_section(&mut self, section: Section) {
        self.brush.queue(section);
    }

    pub fn font(&self, name: &'static str) -> FontId {
        self.fonts[&name.to_string()]
    }
}
