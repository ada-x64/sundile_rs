/// # components
/// a component is any type that is 'static, sized, send and sync
/// 
struct Position {
    x: f32,
    y: f32,
}
struct Rotation {
    yaw: f32,
    pitch: f32,
    roll: f32,
}
pub struct Transform {
    position: Position,
    rotation: Rotation,
}