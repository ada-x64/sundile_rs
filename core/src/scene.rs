use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use sundile_graphics::{Model, ModelInstance};

use crate::Game;

/// Function type for scenes.
/// Initializes the scene. Runs on scene open.
/// TODO: Needs to integrate with an ECS eventually.
pub type SceneFn = fn(SceneBuilder);
/// Map type for scenes.
pub type SceneMap = HashMap<&'static str, SceneFn>;

pub struct SceneBuilder {
    game: Arc<Mutex<Game>>,
}

impl SceneBuilder {
    pub fn new(game: Arc<Mutex<Game>>) -> Self {
        Self { game }
    }

    pub fn new_model_instance(&self, name: &'static str, instance: ModelInstance) {
        let mut game = self.game.lock().unwrap();
        let assets = &mut game.assets;
        assets
            .try_take_asset::<Model>(name)
            .unwrap()
            .instance_cache
            .insert(instance);
        drop(game);
    }
}
