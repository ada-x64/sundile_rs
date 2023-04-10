pub mod prelude {
    pub use sundile_common::*;
    pub use sundile_core::game::Game;
    pub use sundile_graphics::*;
    pub mod egui {
        pub use egui::*;
    }
}
use prelude::egui::*;
pub use prelude::*;

use std::collections::HashMap;

// TODO: Implement a modified egui_wgpu_backend to account for WebGL buffer sizes. cf https://github.com/gfx-rs/wgpu/issues/2573

/// Type alias for [egui_wgpu_backend::RenderPass]. This may change!
pub type DebugGuiRenderer = egui_wgpu_backend::RenderPass;

/// This trait enables libraries to plug-in windows to the debug gui.
pub trait DebugWindow {
    /// Internally calls the Window.show() function.
    fn show(
        &mut self,
        ctx: &Context,
        open: &mut bool,
        renderer: &mut DebugGuiRenderer,
        render_target: &mut RenderTarget,
        game: &mut Game,
    );
}

struct DebugWindowWrapper {
    window: Box<dyn DebugWindow>,
    open: bool,
}

pub struct DebugGui {
    platform: egui_winit::State,
    renderer: DebugGuiRenderer,
    context: Context,
    debug_windows: HashMap<String, DebugWindowWrapper>,

    pub open: bool,
}

impl DebugGui {
    pub fn new<T>(
        render_target: &RenderTarget,
        event_loop: &winit::event_loop::EventLoopWindowTarget<T>,
        debug_windows: HashMap<String, Box<dyn DebugWindow>>,
        open: bool,
    ) -> Self {
        let platform = egui_winit::State::new(event_loop);

        // Why does this have to be Bgra..?
        let renderer =
            DebugGuiRenderer::new(&render_target.device, render_target.texture_format, 1);

        let debug_windows = HashMap::from_iter(debug_windows.into_iter().map(|(name, window)| {
            (
                name,
                DebugWindowWrapper {
                    window,
                    open: false,
                },
            )
        }));

        DebugGui {
            platform,
            renderer,
            context: egui::Context::default(),
            debug_windows,
            open,
        }
    }

    /// Handles the winit event. Will return None if egui wants exclusive control.
    pub fn handle_event<'e, T>(
        &mut self,
        event: winit::event::Event<'e, T>,
    ) -> Option<winit::event::Event<'e, T>> {
        if let winit::event::Event::WindowEvent {
            event: window_event,
            ..
        } = &event
        {
            if self.open && self.platform.on_event(&self.context, window_event).consumed {
                None
            } else {
                Some(event)
            }
        } else {
            Some(event)
        }
    }

    pub fn render(
        &mut self,
        render_target: &mut RenderTarget,
        window: &winit::window::Window,
        game: &mut Game,
        fps: f64,
    ) {
        if !self.open {
            return;
        }
        self.context
            .begin_frame(self.platform.take_egui_input(&window));

        // Iterate through debug windows...
        SidePanel::left("window_picker").show(&self.context, |ui| {
            ui.label("sundile 0.1.0");
            ui.label(format!("{:.2} fps", fps));
            ui.label("(press F5 to toggle this GUI)");
            ui.label("(press ESC to quit)");
            ui.separator();

            ScrollArea::vertical().show(ui, |ui| {
                for (name, wrapper) in &mut self.debug_windows {
                    if ui.button(name).clicked() {
                        wrapper.open = true;
                    }
                    if wrapper.open {
                        wrapper.window.show(
                            &self.context,
                            &mut wrapper.open,
                            &mut self.renderer,
                            render_target,
                            game,
                        )
                    }
                }
            })
        });

        let output = self.context.end_frame();
        let paint_jobs = self.context.tessellate(output.shapes);

        // NOTE: Repainting this every frame no matter what may be a performance issue.
        // See output.needs_repaint
        let (device, queue, encoder, color_view) = (
            &render_target.device,
            &render_target.queue,
            render_target
                .encoder
                .as_mut()
                .expect("Could not get encoder!"),
            render_target
                .color_view
                .as_ref()
                .expect("Could not get color view!"),
        );
        self.renderer
            .add_textures(device, queue, &output.textures_delta)
            .expect("Could not add textures to debug gui!");
        let size = window.inner_size();
        let screen_descriptor = egui_wgpu_backend::ScreenDescriptor {
            physical_width: size.width,
            physical_height: size.height,
            scale_factor: window.scale_factor() as f32,
        };
        self.renderer
            .update_buffers(device, queue, &paint_jobs, &screen_descriptor);
        self.renderer
            .execute(encoder, color_view, &paint_jobs, &screen_descriptor, None)
            .expect("Could not render debug_gui!");

        self.platform
            .handle_platform_output(window, &self.context, output.platform_output);
    }
}
