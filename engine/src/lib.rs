// internal modules
pub mod builders;
pub mod debug_gui;

// exports
pub mod prelude {
    pub mod core {
        pub use sundile_core::*;
    }
    pub mod graphics {
        pub use sundile_graphics::*;
    }
    pub mod assets {
        pub use sundile_assets::*;
    }
    pub use self::core::*;
    pub use crate::builders::*;
    pub use sundile_common::*;
}
use egui_winit::winit::event::VirtualKeyCode;
use log::*;
pub use prelude::*;
use wasm_bindgen::prelude::wasm_bindgen;
use winit::window::*;

//NOTE: Because this is wasm_bindgen, it *cannot* have a lifetime or type parameter!
#[wasm_bindgen]
pub struct Engine {
    event_loop: EventLoop<()>,
    window: Window,
    render_target: RenderTarget,
    assets: AssetTypeMap,
    scene_map: SceneMap,
    debug_gui: DebugGui,
}
#[wasm_bindgen]
impl Engine {
    /// Runs the game.
    /// Note, this hands execution of the main thread over to winit, so make sure this is the last thing you call!
    pub fn run(self) {
        let (event_loop, window, mut render_target, assets, scene_map, mut debug_gui) = (
            self.event_loop,
            self.window,
            self.render_target,
            self.assets,
            self.scene_map,
            self.debug_gui,
        );

        let mut game = Game::new(&render_target, assets, scene_map, None, debug_gui.open);
        let mut fps = 0.0;
        let mut timer = time::Timer::new();
        let mut input = Input::new();

        event_loop.run(
            move |event, _, control_flow| match debug_gui.handle_event(event) {
                Some(event) => {
                    if input.update(&event) {
                        if input.key_pressed(VirtualKeyCode::F5) {
                            debug_gui.open = !debug_gui.open;
                            game.paused = debug_gui.open;
                        }
                        if input.key_pressed(VirtualKeyCode::Escape) {
                            *control_flow = winit::event_loop::ControlFlow::Exit;
                            warn!("Exiting");
                        }
                        game.handle_input(&input);

                        let dt = timer.elapsed();
                        timer.start();

                        let smoothing = 0.9;
                        if dt.as_secs() != 0.0 {
                            fps = fps * smoothing + (1.0 - smoothing) / (dt.as_secs());
                        }
                        game.update(dt);

                        render_target.begin_frame();
                        game.render(&mut render_target);
                        debug_gui.render(&mut render_target, &window, &mut game, fps);
                        render_target.end_frame();

                        input.step();
                    }
                }
                None => {}
            },
        );
    }
}
