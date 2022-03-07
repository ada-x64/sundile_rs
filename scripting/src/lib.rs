pub mod components;
// pub mod scene;

#[test]
fn test_serialize() {

    use legion::{*, serialize::*};
    use components::*;

    let mut world = World::default();
    world.push((
        Transform::default(),
        Model {name: "test-cube".to_string()},
    ));

    let mut registry = Registry::<String>::default();
    components::register(&mut registry);

    let ron = ron::ser::to_string_pretty(&world.as_serializable(
        legion::any(),
        &registry,
    &Canon::default()
        ),
        ron::ser::PrettyConfig::new()
    ).unwrap();

    std::fs::write("./test.ron", ron).unwrap();

}

#[test]
fn test_deserialize() {
    use legion::{*, serialize::*};
    use components::*;

    let data = ron::from_str::<Vec<dyn Component>>(include_str!("./example_scene.ron")).unwrap();

}