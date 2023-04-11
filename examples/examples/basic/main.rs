use sundile::*;

fn main() {
    let engine = sundile::EngineBuilder::new()
        .with_window(WindowBuilder::new().with_title("Sundile"))
        .with_debug_gui(DebugGuiBuilder::new().with_open(true))
        .build();

    engine.run();
}
