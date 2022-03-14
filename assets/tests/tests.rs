use sundile_assets::prelude::*;
use sundile_graphics::prelude::*;

#[test]
fn test_all() {
    Serializer::new()
        .with_asset_directory("C:/dev/Quell/sundile_rs/assets/tests/assets")
        .with_out_path("C:/dev/Quell/sundile_rs/assets/tests/")
        .with_mapper("models",ModelMapper::new())
        .with_mapper("shaders",ShaderMapper::new())
        .serialize();

    let hrt = futures::executor::block_on(
        HeadlessRenderTarget::new(false, None)
    );
    let bin = std::fs::read("C:/dev/Quell/sundile_rs/assets/tests/data.bin").unwrap();
    let map = Deserializer::new()
        .with_mapper("models",ModelMapper::new())
        .with_mapper("shaders",ShaderMapper::new())
        .with_panic(true)
        .deserialize(&bin[..], &hrt);

    assert!(map.try_get_asset::<&str, Model>("models", "cube").is_ok());
    assert!(map.try_get_asset::<&str, wgpu::ShaderModule>("shaders", "passthrough").is_ok());

    let err = map.try_get_asset::<&str, Model>("nonexistent_type", "nonexist_item");
    assert!(err.is_err());
    dbg!(err.unwrap_err());

    let err = map.try_get_asset::<&str, Model>("models", "nonexist_item");
    assert!(err.is_err());
    dbg!(err.unwrap_err());

    let err = map.try_get_asset::<&str, wgpu::ShaderModule>("models", "cube");
    assert!(err.is_err());
    dbg!(err.unwrap_err());
}