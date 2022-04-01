pub use sundile_scripting::prelude::*;
pub use sundile_graphics::prelude::*;
pub use sundile_assets::*;
pub use crate::debug_gui::*;
pub use crate::Engine;
pub use winit::{window::WindowBuilder, event_loop::EventLoop};
use std::collections::HashMap;

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
    /// Creates a new EngineBuilder.
    pub fn new() -> Self {
        Self {
            window_builder: None,
            render_target_builder: None,
            asset_typemap_builder: None,
            scene_map_builder: None,
            debug_gui_builder: None,
            asset_builders: vec![],
        }
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
        self.scene_map_builder = Some(self.scene_map_builder.unwrap_or(SceneMapBuilder::new()).combine(scene_map_builder));
        self
    }
    /// Adds a debug_gui interface. Tip: Use DebugGuiBuilder.
    pub fn with_debug_gui(mut self, debug_gui_builder: DebugGuiBuilder<'a>) -> Self {
        self.debug_gui_builder = Some(self.debug_gui_builder.unwrap_or(DebugGuiBuilder::new()).combine(debug_gui_builder));
        self
    }
    /// Adds an asset builder, which will run its build function on [EngineBuilder::build]
    pub fn with_asset_builder(mut self, asset_builder: impl AssetBuilder + 'a) -> Self {
        self.asset_builders.push(Box::new(asset_builder));
        self
    }
    /// Builds the game engine
    pub fn build(self) -> Engine<'a> {

        let event_loop = EventLoop::new();
        let window = self.window_builder.unwrap_or(WindowBuilder::new()).build(&event_loop).expect("Unable to build window!");
        let render_target = self.render_target_builder.unwrap_or(RenderTargetBuilder::new(None, false)).build(&window);
        let mut assets = self.asset_typemap_builder.unwrap_or(AssetTypeMapBuilder::new()).build(&render_target);
        let debug_gui = self.debug_gui_builder.unwrap_or(DebugGuiBuilder::new()).build(&render_target, &window);
        let scene_map = self.scene_map_builder.unwrap_or(SceneMapBuilder::new()).build();

        for asset_builder in self.asset_builders {
            asset_builder.build(&render_target, &mut assets);
        }

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
    pub fn with_open_status(mut self, open: bool) -> Self {
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
    pub(crate) fn build(self, render_target: &RenderTarget, window: &winit::window::Window,) -> DebugGui<'a> {
        DebugGui::new(render_target, window, self.debug_windows, self.open)
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
        self.map
    }
}

/// Type alias for [sundile_assets::Deserializer]
pub type Deserializer<'a> = sundile_assets::Deserializer<'a>;

/// Builder type for including assets.
pub struct AssetTypeMapBuilder<'a> {
    map: AssetTypeMap,
    deserializer: Option<Deserializer<'a>>,
    bin: Option<&'a [u8]>
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
    pub fn with_asset<T>(mut self, name: &'a str, asset: T) -> Self where T: 'static {
        self.map.try_insert_asset(name, asset).unwrap();
        self
    }
    
    /// Adds a [Deserializer] and the data to be deserialized. (Tip: use [include_bytes!])
    pub fn with_deserializer(mut self, deserializer: Deserializer<'a>, bin: &'a[u8]) -> Self {
        self.deserializer = Some(deserializer);
        self.bin = Some(bin);
        self
    }
    /// Builds the AssetTypeMap
    pub(crate) fn build(mut self, render_target: &RenderTarget) -> AssetTypeMap {
        match self.deserializer {
            Some(de) => {
                self.map.try_combine(de.deserialize(self.bin.unwrap(), render_target)).unwrap();
                self.map
            }
            None => {
                self.map
            }
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
            enable_tracing
        }
    }
    pub(crate) fn build(self, window: &winit::window::Window) -> RenderTarget {
        futures::executor::block_on(
            RenderTarget::new(window, self.enable_tracing, self.label)
        )
    }
}