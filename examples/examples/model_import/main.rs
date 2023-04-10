use sundile::{AssetTypeMapBuilder, Deserializer, WindowBuilder};

fn main() {
    let engine = sundile::EngineBuilder::new()
        .with_window(WindowBuilder::new().with_title("Sundile"))
        .with_assets(
            AssetTypeMapBuilder::new()
                .with_deserializer(Deserializer::default(), include_bytes!("data.bin")),
        )
        .build();

    engine.run();
}
