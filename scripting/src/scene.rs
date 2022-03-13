// use legion::{*, World, serialize::Canon};
use std::collections::HashMap;

/// Function type for scenes.
pub type SceneFn = fn(&mut legion::World);
/// Map type for scenes.
pub type SceneMap = HashMap<&'static str, SceneFn>;

// struct SceneWrapper {
//     map: HashMap<String, World>,
// }
// impl SceneWrapper {
//     pub fn new() -> Self { Self {map: HashMap::new()} }
//     pub fn load_scene(&self, data: &str) {
//         // let scene = ron::from_str(data).unwrap();

//     }

//     pub fn serialize(&self, registry: &mut Registry<String>, name: String) {
//         //TODO: Move this somewhere else.
//         // let mut registry = legion::Registry::default();
//         // registry.register_auto_mapped::<String>();
//         &self.map[&name].as_serializable(
//             legion::any(),
//             &registry,
//             &legion::serialize::Canon::default()
//         );
//     }
// }