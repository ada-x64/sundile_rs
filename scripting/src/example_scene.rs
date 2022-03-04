use legion::*;

fn export() {
    let world = World::default();
    world.push(("some components..."));

    //TODO: Move this somewhere else.
    let mut registry = legion::Registry::default();
    registry.register_auto_mapped::<String>();


    world.as_serializable(
        legion::any(),
        registry,
        entity_serializer //bincode
    );
}