// internal modules
mod debug_gui;
pub mod builders;
use debug_gui::*;
use builders::*;

// internal crates
use sundile_core::prelude::*;
use sundile_graphics::prelude::*;
use sundile_assets::prelude::*;
use sundile_scripting::prelude::*;

// external crates
use winit::{event::*, event_loop::*, window::*};
use std::time::*;

pub struct Engine {
    event_loop: EventLoop<()>,
    window: Window,
    render_target: RenderTarget,
    assets: Assets,
    scene_map: SceneMap,
    debug_gui: DebugGui,
}
impl Engine {
    /// Runs the game.
    /// Note that this hands control of the main thread to winit. Be sure this is the last thing you call!
    pub fn run(mut self) -> () {
        let (window, render_target, assets, scene_map, debug_gui) = (
            self.window,
            self.render_target,
            self.assets,
            self.scene_map,
            self.debug_gui
        );

        let event_loop = EventLoop::new();
        let mut game = game::Game::new(&render_target, &mut assets, scene_map, None, !debug_gui.open);
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
                    
                    //TODO: Thread this process
                    render_target.begin_frame();
                    game.render(&mut render_target, &assets);
                    debug_gui.render(&mut render_target, &window, &game);
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
/// Builder for the game engine. 
pub struct EngineBuilder<'a> {
    window_builder: Option<WindowBuilder>,
    render_target: Option<RenderTarget>,
    assets_bin: Option<&'static [u8]>,
    asset_typemap_builder: Option<AssetTypeMapBuilder<'a>>,
    scene_map_builder: Option<SceneMapBuilder>,
    debug_gui_builder: Option<DebugGuiBuilder>,
}
impl<'a> EngineBuilder<'a> {
    /// Creates a new EngineBuilder.
    pub fn new() -> Self {
        Self {
            window_builder: None,
            render_target: None,
            assets_bin: None,
            asset_typemap_builder: None,
            scene_map_builder: None,
            debug_gui_builder: None,
        }
    }
    /// Overrides the default window. For more info see [winit::WindowBuilder]
    pub fn with_window(mut self, window_builder: WindowBuilder) -> Self {
        self.window_builder = Some(window_builder);
        self
    }
    /// Sets a custom render target.
    pub fn with_render_target(mut self, render_target: RenderTarget) -> Self {
        self.render_target = Some(render_target);
        self
    }
    /// Manually adds an [AssetsBuilder], which will load assets either statically or at runtime.
    pub fn with_asset_map_builder(mut self, assets_builder: AssetTypeMapBuilder<'a>) -> Self {
        self.asset_typemap_builder = Some(self.asset_typemap_builder.unwrap_or(AssetTypeMapBuilder::new()).with_builder(assets_builder));
        self
    }
    /// Manually adds a [SceneMapBuilder], which will add scenes at build time.
    /// Note: This will overwrite any currently existing SceneMapBuilder, so call this only once.
    pub fn with_scene_map(mut self, scene_map_builder: SceneMapBuilder) -> Self {
        self.scene_map_builder = Some(self.scene_map_builder.unwrap_or(SceneMapBuilder::new()).with_builder(scene_map_builder));
        self
    }
    /// Adds a debug_gui interface. Tip: Use DebugGuiBuilder.
    pub fn with_debug_gui(mut self, debug_gui_builder: DebugGuiBuilder) -> Self {
        self.debug_gui_builder = Some(debug_gui_builder);
        self
    }
    /// Builds the game engine
    pub fn build(mut self) -> Engine {

        let event_loop = EventLoop::new();
        let window = self.window_builder.unwrap_or(WindowBuilder::new()).build(&event_loop).expect("Unable to build window!");
        let render_target = self.render_target.unwrap_or(
            futures::executor::block_on(RenderTarget::new(&window, false, Some("Default Render Target"))),
        );
        let assets = self.asset_typemap_builder.unwrap_or(AssetTypeMapBuilder::new()).build(&render_target);
        let debug_gui = self.debug_gui_builder.unwrap_or(DebugGuiBuilder::new()).build(&render_target, &window);
        let scene_map = self.scene_map_builder.unwrap_or(SceneMapBuilder::new()).build();

        Engine {
            event_loop,
            window,
            render_target,
            assets,
            debug_gui,
            scene_map,
        }
    }
}