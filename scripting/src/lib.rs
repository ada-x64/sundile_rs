use legion::*;

pub mod components;
pub mod example_scene;

pub trait Component : 'static + Sized + Send + Sync {}

pub struct Scene {
    entities: Vec<Box<dyn Component>>
}
impl Scene {
    pub fn load(&self, world: &legion::World) {
        
    }
}