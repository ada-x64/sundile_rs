use sundile::{
    AssetTypeMapBuilder, Deserializer, ModelInstance, SceneBuilder, SceneMapBuilder, WindowBuilder,
};

fn default_scene(b: SceneBuilder) {
    b.new_model_instance("test_cube", ModelInstance::at_origin())
}

fn main() {
    let engine = sundile::EngineBuilder::new()
        .with_window(WindowBuilder::new().with_title("Sundile"))
        .with_assets(
            AssetTypeMapBuilder::new()
                .with_deserializer(Deserializer::default(), include_bytes!("data.bin")),
        )
        .with_scenes(SceneMapBuilder::new().with_scene("default", default_scene))
        .build();

    engine.run();
}
