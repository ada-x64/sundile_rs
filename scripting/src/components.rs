/// # components
/// a component is any type that is 'static, sized, send and sync
// TODO: Split this into submodules when the time comes.
// TODO: Figure out how to impl IntoComponent for structs with nested structs.

use serde::*;

pub trait Component : 'static + Sized + Send + Sync {}

/// temporary! Probably make a macro #[derive(Component)] that has a register function
pub fn register(registry: &mut legion::Registry<String>) {
    registry.register::<Transform>("Transform".to_string());
    registry.register::<Model>("Model".to_string());
}

/// Transform struct that determines the position and rotation of an entity.
#[derive(Serialize, Deserialize)]
pub struct Transform {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,
}
impl Default for Transform {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0)
    }
}
impl Transform {
    pub fn new(x:f32, y:f32, z:f32, yaw:f32, pitch:f32, roll:f32) -> Self {
        Self {x, y, z, yaw, pitch, roll}
    }
}

/// Model struct. Points to the name of an OBJ model.
#[derive(Serialize, Deserialize)]
pub struct Model {
    pub name: String,
}
