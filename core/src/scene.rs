use std::collections::HashMap;

/// Function type for scenes.
/// TODO: Needs to integrate with an ECS eventually.
pub type SceneFn = fn();
/// Map type for scenes.
pub type SceneMap = HashMap<&'static str, SceneFn>;
