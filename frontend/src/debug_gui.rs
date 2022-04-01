pub mod prelude {
    pub use sundile_graphics::prelude::*;
    pub use sundile_core::game::Game;
    pub mod egui {pub use egui::*;}
}
pub use prelude::*;
use prelude::egui::*;

use std::collections::HashMap;

/// Type alias for [egui_wgpu_backend::RenderPass]. This may change!
pub type DebugGuiRenderer = egui_wgpu_backend::RenderPass;

/// This trait enables libraries to plug-in windows to the debug gui.
pub trait DebugWindow {
    /// Internally calls the Window.show() function.
    fn show(&mut self, ctx: &Context, open: &mut bool, renderer: &mut DebugGuiRenderer, render_target: &mut RenderTarget, game: &mut Game);
}

struct DebugWindowWrapper {
    window: Box<dyn DebugWindow>,
    open: bool,
}

pub struct DebugGui<'a> {
    platform: egui_winit::State,
    renderer: DebugGuiRenderer,
    context: Context,
    debug_windows: HashMap<&'a str, DebugWindowWrapper>,

    pub open: bool,
}

impl<'a> DebugGui<'a> {
    pub fn new(render_target: &RenderTarget, window: &winit::window::Window, debug_windows: HashMap<&'a str, Box<dyn DebugWindow>>, open: bool) -> Self {
        let platform = egui_winit::State::new(render_target.device.limits().max_texture_dimension_2d as usize, &window);

        // Why does this have to be Bgra..?
        let renderer = DebugGuiRenderer::new(&render_target.device, wgpu::TextureFormat::Bgra8UnormSrgb, 1);

        let debug_windows = HashMap::from_iter(
            debug_windows.into_iter().map(|(name, window)| {
                (name, DebugWindowWrapper { window, open: false,})
            })
        );

        DebugGui {
            platform,
            renderer,
            context: egui::Context::default(),
            debug_windows,
            open,
        }
    }

    /// Handles the winit event. If this returns true, the event can be reused.
    pub fn handle_event<'e>(&mut self, event: winit::event::WindowEvent<'e>, control_flow: &mut winit::event_loop::ControlFlow) -> Option<winit::event::WindowEvent<'e>> {
        use winit::{event::*, event_loop::*};
        let mut exclusive_use = false;
        match event {
            WindowEvent::CloseRequested => {
                *control_flow = ControlFlow::Exit;
            },
            WindowEvent::KeyboardInput {input, ..} => {
                if input.state == ElementState::Released {
                    match input.virtual_keycode {
                        Some(code) => {
                            match code {
                                VirtualKeyCode::F5 => {
                                    self.open = !self.open;
                                    exclusive_use = true;
                                }
                                VirtualKeyCode::Escape => {
                                    *control_flow = ControlFlow::Exit;
                                    exclusive_use = true;
                                }
                                _ => {}
                            }
                        }
                        None => {}
                    }
                }
            },
            _ => {
            }
        }
        if self.platform.on_event(&self.context, &event) && !exclusive_use {
            Some(event)
        }
        else {
            None
        }
    }

    pub fn render(&mut self, render_target: &mut RenderTarget, window: &winit::window::Window, game: &mut Game) {
        //
        // Send to egui
        //
        self.context.begin_frame(self.platform.take_egui_input(&window));

        // Iterate through debug windows...
        SidePanel::left("window_picker").show(&self.context, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                for (name, wrapper) in &mut self.debug_windows {
                    if ui.button(*name).clicked() {
                        wrapper.open = true;
                    }
                    if wrapper.open {
                        wrapper.window.show(&self.context, &mut wrapper.open, &mut self.renderer, render_target, game)
                    }
                }
            })
        });

        let output = self.context.end_frame();
        let paint_jobs = self.context.tessellate(output.shapes);

        // NOTE: Repainting this every frame no matter what may be a performance issue.
        // See output.needs_repaint
        let (
            device,
            queue,
            encoder,
            color_view,
        ) = (
            &render_target.device,
            &render_target.queue,
            render_target.encoder.as_mut().expect("Could not get encoder!"),
            render_target.color_view.as_ref().expect("Could not get color view!"),
        );
        self.renderer.add_textures(device, queue, &output.textures_delta).expect("Could not add textures to debug gui!");
        let size = window.inner_size();
        let screen_descriptor = egui_wgpu_backend::ScreenDescriptor {
            physical_width: size.width,
            physical_height: size.height,
            scale_factor: window.scale_factor() as f32,
        };
        self.renderer.update_buffers(device, queue, &paint_jobs, &screen_descriptor);
        self.renderer.execute(
            encoder,
            color_view,
            &paint_jobs,
            &screen_descriptor,
            None,
        ).expect("Could not render debug_gui!");

        self.platform.handle_platform_output(window, &self.context, output.platform_output);
    }
}