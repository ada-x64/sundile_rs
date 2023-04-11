use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use sundile_assets::AssetTypeMap;
use sundile_graphics::{Model, ModelInstance, TextBlock, TextBlockInstance};

/// Function type for scenes.
/// Initializes the scene. Runs on scene open.
/// TODO: Needs to integrate with an ECS eventually.
pub type SceneFn = fn(SceneBuilder);
/// Map type for scenes.
pub type SceneMap = HashMap<&'static str, SceneFn>;

pub struct SceneBuilder {
    pub assets: Arc<Mutex<AssetTypeMap>>,
}

impl SceneBuilder {
    pub fn new(assets: Arc<Mutex<AssetTypeMap>>) -> Self {
        Self { assets }
    }

    // TODO: These intsancaing functions should be a single function.
    pub fn new_model_instance(&self, name: &'static str, instance: ModelInstance) {
        let mut assets = self.assets.lock().unwrap();
        // TODO: This is atrocious.
        let mut model = assets.try_take_asset::<Model>(name).unwrap();
        model.instance_cache.insert(instance);
        assets.try_insert_asset(name, model).unwrap();
    }

    pub fn new_text_instance(&self, name: &'static str, instance: TextBlockInstance) {
        let mut assets = self.assets.lock().unwrap();
        let mut text = assets.try_take_asset::<TextBlock>(name).unwrap();
        text.instance_cache.push(instance);
        assets.try_insert_asset(name, text).unwrap();
    }
}
