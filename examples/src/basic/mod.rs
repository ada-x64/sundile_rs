use sundile::WindowBuilder;

#[allow(dead_code)]
pub fn doit(window_builder: WindowBuilder) {
    let engine = sundile::EngineBuilder::new()
        .with_window(window_builder.with_title("Sundile - Basic Example"))
        .build();

    engine.run();
}
