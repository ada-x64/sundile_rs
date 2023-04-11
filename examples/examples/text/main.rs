use sundile::{
    wgpu_glyph::{BuiltInLineBreaker, HorizontalAlign, Layout, VerticalAlign},
    EngineBuilder, FontSpecifier, SceneBuilder, SceneMapBuilder, TextBlock, TextBlockInstance,
    WindowBuilder,
};

fn default_scene(b: SceneBuilder) {
    {
        let text = TextBlock::new(String::from("Hello, text!"));
        let lock = b.assets.lock();
        let mut assets = lock.unwrap();
        assets.try_insert_asset("text test", text).unwrap();
    }
    b.new_text_instance(
        "text test",
        TextBlockInstance {
            x: 0.5,
            y: 0.5,
            relative_position: true,
            font: Some(FontSpecifier {
                name: "regular".into(),
                size: 128.0,
            }),
            layout: Some(Layout::SingleLine {
                h_align: HorizontalAlign::Center,
                v_align: VerticalAlign::Center,
                line_breaker: BuiltInLineBreaker::default(),
            }),
        },
    );
}

fn main() {
    EngineBuilder::new()
        .with_window(WindowBuilder::new().with_title("Sundile"))
        .with_scenes(SceneMapBuilder::new().with_scene("default", default_scene))
        .build()
        .run();
}
