use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use sundile_assets::AssetTypeMap;
use sundile_graphics::{Model, ModelInstance};

/// Function type for scenes.
/// Initializes the scene. Runs on scene open.
/// TODO: Needs to integrate with an ECS eventually.
pub type SceneFn = fn(SceneBuilder);
/// Map type for scenes.
pub type SceneMap = HashMap<&'static str, SceneFn>;

pub struct SceneBuilder {
    assets: Arc<Mutex<AssetTypeMap>>,
}

impl SceneBuilder {
    pub fn new(assets: Arc<Mutex<AssetTypeMap>>) -> Self {
        Self { assets }
    }

    pub fn new_model_instance(&self, name: &'static str, instance: ModelInstance) {
        let mut assets = self.assets.lock().unwrap();
        // TODO: This is atrocious.
        let mut model = assets.try_take_asset::<Model>(name).unwrap();
        model.instance_cache.insert(instance);
        assets.try_insert_asset(name, model).unwrap();
    }
}
