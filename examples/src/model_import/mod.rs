use std::f32::consts::PI;

use cgmath::{Euler, Rad, Vector3};
use sundile::{
    AssetTypeMapBuilder, Deserializer, EngineBuilder, ModelInstance, SceneBuilder, SceneMapBuilder,
    WindowBuilder,
};

fn default_scene(b: SceneBuilder) {
    for i in 0..10 {
        for j in 0..10 {
            let x = i as f32;
            let z = j as f32;
            b.new_model_instance(
                "cube",
                ModelInstance::new(
                    Vector3::new(x * 5., 0.0, z * 5.),
                    Euler::new(Rad(z * 5. * PI / 10.0), Rad(x * 2. * PI / 10.), Rad(0.0)).into(),
                ),
            )
        }
    }
}

/// NOTE: In order to run this example you will need to generate the precompiled assets.
/// sundile_serialize_assets -i ./src/model_import/assets -o ./src/model_import -m
#[allow(dead_code)]
pub fn doit() {
    EngineBuilder::new()
        .with_window(WindowBuilder::new().with_title("Sundile"))
        .with_assets(
            AssetTypeMapBuilder::new()
                .with_deserializer(Deserializer::default(), include_bytes!("data.bin")),
        )
        .with_scenes(SceneMapBuilder::new().with_scene("default", default_scene))
        .build()
        .run();
}
