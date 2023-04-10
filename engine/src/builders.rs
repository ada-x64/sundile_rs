pub use crate::debug_gui::*;
pub use crate::Engine;
pub use log;
use log::debug;
use std::collections::HashMap;
pub use sundile_assets::*;
pub use sundile_common::*;
use sundile_core::SceneFn;
use sundile_core::SceneMap;
pub use sundile_graphics::*;
pub use winit::{event_loop::EventLoop, window::WindowBuilder};

/// Builder for the game engine.
pub struct EngineBuilder<'a> {
    window_builder: Option<WindowBuilder>,
    render_target_builder: Option<RenderTargetBuilder<'a>>,
    asset_typemap_builder: Option<AssetTypeMapBuilder<'a>>,
    scene_map_builder: Option<SceneMapBuilder>,
    debug_gui_builder: Option<DebugGuiBuilder<'a>>,
    asset_builders: Vec<Box<dyn AssetBuilder + 'a>>,
}
impl<'a> EngineBuilder<'a> {
    /// Creates a new EngineBuilder and initializes Sundile. This should be the first thing you call.
    pub fn new() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            env_logger::init();
        }
        #[cfg(target_arch = "wasm32")]
        {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("could not initialize logger");
        }

        Self {
            window_builder: None,
            render_target_builder: None,
            asset_typemap_builder: None,
            scene_map_builder: None,
            debug_gui_builder: None,
            asset_builders: vec![],
        }
    }
    /// Sets the log level.
    pub fn with_log_level(self, level_filter: log::LevelFilter) -> Self {
        log::set_max_level(level_filter);
        self
    }
    /// Overrides the default window. For more info see [winit::WindowBuilder]
    pub fn with_window(mut self, window_builder: WindowBuilder) -> Self {
        self.window_builder = Some(window_builder);
        self
    }
    /// Sets a custom render target.
    pub fn with_render_target(mut self, render_target_builder: RenderTargetBuilder<'a>) -> Self {
        self.render_target_builder = Some(render_target_builder);
        self
    }
    /// Adds an [AssetTypeMapBuilder], which will load assets either statically or at runtime.
    pub fn with_assets(mut self, assets_builder: AssetTypeMapBuilder<'a>) -> Self {
        self.asset_typemap_builder = Some(assets_builder);
        self
    }
    /// Adds a [SceneMapBuilder], which will add scenes at build time.
    pub fn with_scenes(mut self, scene_map_builder: SceneMapBuilder) -> Self {
        self.scene_map_builder = Some(
            self.scene_map_builder
                .unwrap_or(SceneMapBuilder::new())
                .combine(scene_map_builder),
        );
        self
    }
    /// Adds a debug_gui interface. Tip: Use DebugGuiBuilder.
    pub fn with_debug_gui(mut self, debug_gui_builder: DebugGuiBuilder<'a>) -> Self {
        self.debug_gui_builder = Some(
            self.debug_gui_builder
                .unwrap_or(DebugGuiBuilder::new())
                .combine(debug_gui_builder),
        );
        self
    }
    /// Adds an asset builder, which will run its build function on [EngineBuilder::build]
    pub fn with_asset_builder(mut self, asset_builder: impl AssetBuilder + 'a) -> Self {
        self.asset_builders.push(Box::new(asset_builder));
        self
    }
    /// Builds the game engine
    pub fn build(self) -> Engine {
        debug!("Building engine...");
        let event_loop = EventLoop::new();
        let window = self
            .window_builder
            .unwrap_or(WindowBuilder::new())
            .build(&event_loop)
            .expect("Unable to build window!");
        #[cfg(target_arch = "wasm32")]
        {
            // Append the canvas to the document body.
            use winit::platform::web::WindowExtWebSys;
            let web_window = web_sys::window().unwrap();
            let doc = web_window.document().unwrap();
            let body = doc.body().unwrap();
            body.append_child(&window.canvas()).unwrap();
            debug!("Canvas created.");
        }

        let render_target = self
            .render_target_builder
            .unwrap_or(RenderTargetBuilder::new(None, false))
            .build(&window);
        let mut assets = self
            .asset_typemap_builder
            .unwrap_or(AssetTypeMapBuilder::new())
            .build(&render_target);
        let debug_gui = self
            .debug_gui_builder
            .unwrap_or(DebugGuiBuilder::new())
            .build(&render_target, &window, &event_loop);
        let scene_map = self
            .scene_map_builder
            .unwrap_or(SceneMapBuilder::new())
            .build();

        for asset_builder in self.asset_builders {
            debug!("Building Ext Asset");
            asset_builder.build(&render_target, &mut assets);
        }
        debug!("...Engine build finished");

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

/// Builder for DebugGui. Takes structs that implement DebugWindow and adds them to a list to be used internally.
/// TODO: Add option to remove DebugGui from the build.
pub struct DebugGuiBuilder<'a> {
    debug_windows: HashMap<&'a str, Box<dyn DebugWindow>>,
    open: bool,
}
impl<'a> DebugGuiBuilder<'a> {
    /// Creates a DebugGuiBuilder with no debug windows and default settings.
    pub fn new() -> Self {
        Self {
            debug_windows: HashMap::new(),
            open: false,
        }
    }
    /// Adds an externally defined debug window to the gui.
    pub fn with_window(mut self, name: &'a str, window: impl DebugWindow + 'static) -> Self {
        self.debug_windows.insert(name, Box::new(window));
        self
    }
    /// Sets whether the debug gui is open at startup.
    pub fn with_open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }
    /// Combines two builders. The open status is set by ORing the values passed in.
    pub fn combine(mut self, other: Self) -> Self {
        self.debug_windows.extend(other.debug_windows);
        self.open = self.open || other.open;
        self
    }
    /// Builds the debug gui. Should only be used internally.
    pub(crate) fn build<T>(
        self,
        render_target: &RenderTarget,
        window: &winit::window::Window,
        event_loop: &winit::event_loop::EventLoopWindowTarget<T>,
    ) -> DebugGui {
        debug!("Building DebugGui");
        let debug_windows = HashMap::from_iter(
            self.debug_windows
                .into_iter()
                .map(|(key, value)| (key.to_string(), value)),
        );
        DebugGui::new(render_target, window, event_loop, debug_windows, self.open)
    }
}

/// Creates SceneMaps. To be passed in to EngineBuilder.
pub struct SceneMapBuilder {
    map: SceneMap,
}
impl SceneMapBuilder {
    /// Creates a new SceneMapBuilder with default settings.
    pub fn new() -> Self {
        Self {
            map: SceneMap::new(),
        }
    }
    /// Adds a scene to the map.
    pub fn with_scene(mut self, name: &'static str, func: SceneFn) -> Self {
        self.map.insert(name, func);
        self
    }
    /// Consumes another builder and combines its data.
    pub fn combine(mut self, other: Self) -> Self {
        self.map.extend(other.map);
        self
    }
    /// Used to build the SceneMap. Should only be used internally.
    pub(crate) fn build(self) -> SceneMap {
        debug!("Building SceneMap");
        self.map
    }
}

/// Type alias for [sundile_assets::Deserializer]
pub type Deserializer<'a> = sundile_assets::Deserializer<'a>;

/// Builder type for including assets.
pub struct AssetTypeMapBuilder<'a> {
    map: AssetTypeMap,
    deserializer: Option<Deserializer<'a>>,
    bin: Option<&'a [u8]>,
}
impl<'a> AssetTypeMapBuilder<'a> {
    /// Creates a new AssetTypeMapBuilder with default options.
    pub fn new() -> Self {
        Self {
            map: AssetTypeMap::new(),
            deserializer: None,
            bin: None,
        }
    }
    /// Adds an asset. Will create a category for the associated type if needed.
    pub fn with_asset<T>(mut self, name: &'a str, asset: T) -> Self
    where
        T: 'static,
    {
        self.map.try_insert_asset(name, asset).unwrap();
        self
    }

    /// Adds a [Deserializer] and the data to be deserialized. (Tip: use [include_bytes!])
    pub fn with_deserializer(mut self, deserializer: Deserializer<'a>, bin: &'a [u8]) -> Self {
        self.deserializer = Some(deserializer);
        self.bin = Some(bin);
        self
    }
    /// Builds the AssetTypeMap
    pub(crate) fn build(mut self, render_target: &RenderTarget) -> AssetTypeMap {
        debug!("Building AssetMap");
        match self.deserializer {
            Some(de) => {
                self.map
                    .try_combine(de.deserialize(self.bin.unwrap(), render_target))
                    .unwrap();
                self.map
            }
            None => self.map,
        }
    }
}

/// Implement this trait to add assets at build time.
pub trait AssetBuilder {
    fn build(self: Box<Self>, render_target: &RenderTarget, assets: &mut AssetTypeMap);
}

/// Basic builder for [RenderTarget]s.
pub struct RenderTargetBuilder<'a> {
    label: Option<&'a str>,
    enable_tracing: bool,
}
impl<'a> RenderTargetBuilder<'a> {
    pub fn new(label: Option<&'a str>, enable_tracing: bool) -> Self {
        Self {
            label,
            enable_tracing,
        }
    }
    pub(crate) fn build(self, window: &winit::window::Window) -> RenderTarget {
        debug!("Building RenderTarget");
        futures::executor::block_on(RenderTarget::new(window, self.enable_tracing, self.label))
    }
}
