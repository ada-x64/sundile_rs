// internal modules
pub mod debug_gui;
pub mod builders;
use debug_gui::*;

// internal crates
pub use sundile_core::prelude::*;
use sundile_graphics::prelude::*;
use sundile_assets::prelude::*;
use sundile_scripting::prelude::*;

// external crates
use winit::{event::*, event_loop::*, window::*};
use std::time::*;

// exports
pub mod prelude {
    pub mod core { pub use sundile_core::*; }
    pub mod graphics { pub use sundile_graphics::*; }
    pub mod assets { pub use sundile_assets::*; }
    pub mod scripting { pub use sundile_scripting::*; }
    pub use crate::builders;
    pub use crate::debug_gui;
}
pub use prelude::*;

pub struct Engine<'a> {
    event_loop: EventLoop<()>,
    window: Window,
    render_target: RenderTarget,
    assets: AssetTypeMap,
    scene_map: SceneMap,
    debug_gui: DebugGui<'a>,
}
impl Engine<'static> {
    /// Runs the game.
    /// Note that this hands control of the main thread to winit. Be sure this is the last thing you call!
    pub fn run(self) -> () {
        let (
            event_loop,
            window,
            mut render_target,
            assets,
            scene_map,
            mut debug_gui
        ) = (
            self.event_loop,
            self.window,
            self.render_target,
            self.assets,
            self.scene_map,
            self.debug_gui
        );

        let mut game = Game::new(&render_target, assets, scene_map, None, !debug_gui.open);
        let mut fps = 0.0;
        let mut prev_time = Instant::now();
     
        event_loop.run(move |event, _, control_flow| {
    
            debug_gui.handle_event(&event);
            game.paused = !debug_gui.open;
    
            match event {
                winit::event::Event::MainEventsCleared => {
                            
                    let time = Instant::now();
                    let dt = time - prev_time;
                    prev_time = time;
    
                    let smoothing = 0.9;
                    fps = fps*smoothing + (1.0-smoothing)/dt.as_secs_f64();
                    game.update(dt);
                    
                    render_target.begin_frame();
                    game.render(&mut render_target);
                    debug_gui.render(&mut render_target, &window, &mut game);
                    render_target.end_frame();
                },
                winit::event::Event::WindowEvent {window_id, event}
                    if window_id == window.id() => {
                    match event {
                        WindowEvent::CloseRequested => {
                            *control_flow = ControlFlow::Exit;
                        },
                        //TODO: Implement Input system.
                        WindowEvent::KeyboardInput {input, ..} => {
                            if input.state == ElementState::Released {
                                match input.virtual_keycode {
                                    Some(code) => {
                                        match code {
                                            VirtualKeyCode::F5 => {
                                                debug_gui.open = !debug_gui.open;
                                            }
                                            VirtualKeyCode::Escape => {
                                                *control_flow = ControlFlow::Exit;
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
                },
                winit::event::Event::DeviceEvent {ref event, ..} => {
                    game.handle_input(&event);
                },
                _ => {}
            }
        });
    }
}
