use sundile_scripting::prelude::*;
use sundile_graphics::prelude::*;
use sundile_assets::prelude::*;
use crate::debug_gui::*;

/// Builder for DebugGui. Takes structs that implement DebugWindow and adds them to a list to be used internally.
pub struct DebugGuiBuilder {
    debug_windows: Vec<Box<dyn DebugWindow>>,
    open: bool,
}
impl DebugGuiBuilder {
    /// Creates a DebugGuiBuilder with no debug windows and default settings.
    pub fn new() -> Self {
        Self {
            debug_windows: vec![],
            open: false,
        }
    }
    /// Adds an externally defined debug window to the gui.
    pub fn with_window(mut self, window: impl DebugWindow + 'static) -> Self {
        self.debug_windows.push(Box::new(window));
        self
    }
    /// Sets whether the debug gui is open at startup.
    pub fn with_open_status(mut self, open: bool) -> Self {
        self.open = open;
        self
    }
    /// Builds the debug gui. Should only be used internally.
    pub(crate) fn build(self, render_target: &RenderTarget, window: &winit::window::Window,) -> DebugGui {
        DebugGui::new(render_target, window, self.debug_windows, self.open)
    }
}

/// Creates SceneMaps. To be passed in to EngineBuilder.
/// TODO: Add SceneBuilder
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
    pub fn with_builder(mut self, other: Self) -> Self {
        self.map.extend(other.map);
        self
    }
    /// Used to build the SceneMap. Should only be used internally.
    pub(crate) fn build(self) -> SceneMap {
        self.map
    }
}

pub struct AssetTypeMapBuilder<'a> {
    map: AssetTypeMap<'a>,
    static_assets: Vec<&'a [u8]>,
}
impl<'a> AssetTypeMapBuilder<'a> {
    /// Creates a new AssetTypeMapBuilder with default options.
    pub fn new() -> Self {
        Self {
            map: AssetsMap::new(),
            static_assets: Vec::new(),
        }
    }
    /// Adds an asset type.
    pub fn with_asset(mut self, name: &'a str, asset: impl Asset + 'a) -> Self {
        match self.map.get(name) {
            Some(value) => {
                value.insert(name, Box::new(asset));
            }
            None => {
                let submap = std::collections::HashMap::<&'a str, Box<dyn Asset + 'a>>::new();
                submap.insert(name, Box::new(asset));
                self.map.insert(asset.type_name(), submap);
            }
        }
        self
    }
    pub fn with_static(mut self, static_assets: &'a [u8]) -> Self {
        self.static_assets.push(static_assets);
        self
    }
    /// Consumes another builder and combines its data.
    pub fn with_builder(mut self, other: AssetTypeMapBuilder<'a>) -> Self {
        self.static_assets.extend(other.static_assets);
        self.map.combine(other.map);
        self
    }
    pub fn build(mut self, render_target: &RenderTarget) -> AssetsMap {
        for bin in self.static_assets {
            self.map.combine(sundile_assets::parse(bin, &render_target));
        }
        self.map
    }
}