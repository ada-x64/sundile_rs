use sundile_graphics::prelude::*;
use sundile_core::game::Game;

use winit::{event::*, window::*};
use egui_winit_platform::*;
use egui_wgpu_backend::*;
use std::{time::*, collections::HashMap};

//Ref: https://github.com/hasenbanck/egui_example/blob/master/src/main.rs

/// This trait enables libraries to plug-in windows to the debug gui.
pub trait DebugWindow {
    /// Internally calls the egui::Window.show() function.
    fn show(&mut self, ctx: &egui::CtxRef, open: &mut bool, render_target: &mut RenderTarget, game: &mut Game);
}

struct DebugWindowWrapper {
    window: Box<dyn DebugWindow>,
    open: bool,
}

pub struct DebugGui<'a> {
    platform: Platform,
    render_pass: RenderPass,
    start_time: Instant,
    debug_windows: HashMap<&'a str, DebugWindowWrapper>,
    pub open: bool,
}

impl<'a> DebugGui<'a> {
    pub fn new(render_target: &RenderTarget, window: &Window, debug_windows: HashMap<&'a str, Box<dyn DebugWindow>>, open: bool) -> Self {
        let size = window.inner_size();

        let platform = Platform::new(PlatformDescriptor {
            physical_width: size.width as u32,
            physical_height: size.height as u32,
            scale_factor: window.scale_factor(),
            font_definitions: egui::FontDefinitions::default(),
            style: Default::default(),
        });
        
        let render_pass = RenderPass::new(
            &render_target.device,
            render_target.surface.get_preferred_format(&render_target.adapter).unwrap(),
            1,
        );

        let debug_windows = HashMap::from_iter(
            debug_windows.into_iter().map(|(name, window)| {
                (name, DebugWindowWrapper { window, open: false,})
            })
        );

        DebugGui {
            platform,
            render_pass,
            start_time: Instant::now(),
            debug_windows,
            open,
        }
    }

    pub fn handle_event<T>(&mut self, event: &Event<T>,) {
        self.platform.handle_event(event);
    }

    pub fn render(&mut self, render_target: &mut RenderTarget, window: &Window, game: &mut Game) {
        //
        // Send to egui
        //
        self.platform.update_time(self.start_time.elapsed().as_secs_f64());
        self.platform.begin_frame();

        // Iterate through debug windows...
        egui::SidePanel::left("window_picker").show(&self.platform.context(), |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                for (name, wrapper) in &mut self.debug_windows {
                    if ui.button(*name).clicked() {
                        wrapper.open = true;
                    }
                    if wrapper.open {
                        wrapper.window.show(&self.platform.context(), &mut wrapper.open, render_target, game)
                    }
                }
            })
        });

        let (_output, paint_commands) = self.platform.end_frame(Some(&window));
        let paint_jobs = self.platform.context().tessellate(paint_commands);
        
        //
        // Send to GPU
        //
        let screen_descriptor = ScreenDescriptor {
            physical_width: render_target.config.width,
            physical_height: render_target.config.height,
            scale_factor: window.scale_factor() as f32,
        };


        let (
            device,
            queue,
            encoder,
            color_view,
        ) = (
            &render_target.device,
            &render_target.queue,
            render_target.encoder.as_mut().unwrap(),
            render_target.color_view.as_mut().unwrap(),
        );
        self.render_pass.update_texture(device, queue, &self.platform.context().font_image());
        self.render_pass.update_user_textures(device, queue);
        self.render_pass.update_buffers(device, queue, &paint_jobs, &screen_descriptor);
        self.render_pass
            .execute(
                encoder,
                color_view,
                &paint_jobs,
                &screen_descriptor,
                None, //Some(wgpu::Color{r: 0.0, g: 0.0, b: 0.0, a: 0.5}),
            )
            .unwrap();
    }
}