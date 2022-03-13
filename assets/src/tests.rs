use crate::prelude::*;

#[test]
fn test_serialize() {
    Serializer::default().serialize();
}

#[test]
fn test_deserialize() {
    let hrt = futures::executor::block_on(
        sundile_graphics::render_target::HeadlessRenderTarget::new(false, None)
    );
    let bin = std::fs::read(std::env::current_dir().unwrap().join("../data.bin")).unwrap();
    Deserializer::default().deserialize(&bin[..], &hrt);
}