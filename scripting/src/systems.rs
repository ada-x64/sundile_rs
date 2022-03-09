/// Internal systems to be made into a Schedule, alongside additional systems created for the game bin.
/// 

use legion::*;
#[system(for_all)]
pub fn instancing(transform: &Transform, model: &Model) {
    // Add instances to the instance_cache here.
    // How to get access to it without exposing the Renderer struct?
    todo!()
}