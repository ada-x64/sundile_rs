use sundile::WindowBuilder;

#[allow(dead_code)]
pub fn doit() {
    let engine = sundile::EngineBuilder::new()
        .with_window(WindowBuilder::new().with_title("Sundile"))
        .build();

    engine.run();
}
