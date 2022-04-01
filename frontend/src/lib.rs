// internal modules
pub mod debug_gui;
pub mod builders;
use debug_gui::*;

// external crates
use winit::{event_loop::*, window::*};
use std::time::*;

// exports
pub mod prelude {
    pub mod core { pub use sundile_core::*; }
    pub mod graphics { pub use sundile_graphics::*; }
    pub mod assets { pub use sundile_assets::*; }
    pub mod scripting { pub use sundile_scripting::*; }
    pub use self::core::*;
    pub use crate::builders;
    pub use crate::debug_gui;
}
pub use prelude::*;
use prelude::{assets::*, builders::*};

pub struct Engine<'a> {
    event_loop: EventLoop<()>,
    window: Window,
    render_target: RenderTarget,
    assets: AssetTypeMap,
    scene_map: SceneMap,
    debug_gui: DebugGui<'a>,
}
impl Engine<'static> {

    fn run_internal(self) {
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
                        
                    match debug_gui.handle_event(event, control_flow) {
                        Some(_event) => {
                            //perhaps pass to game here?
                        }
                        _ => {}
                    }
                },
                //TODO : ensure device event is for this window!
                winit::event::Event::DeviceEvent {ref event, ..} => {
                    game.handle_input(&event);
                },
                _ => {}
            }
        });
    }

    /// Runs the game.
    /// Note that this hands control of the main thread to winit. Be sure this is the last thing you call!
    pub fn run(self) -> () {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.run_internal();
        }
        #[cfg(target_arch = "wasm32")]
        {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init().expect("could not initialize logger");
            // Append the canvas to the document body.
            use winit::platform::web::WindowExtWebSys;
    
            let web_window = web_sys::window().unwrap();
            let doc = web_window.document().unwrap();
            let body = doc.body().unwrap();
            body.append_child(&self.window.canvas()).unwrap();
    
            self.run_internal();
        }

    }
}
