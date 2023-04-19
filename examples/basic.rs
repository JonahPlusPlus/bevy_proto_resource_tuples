use bevy::prelude::*;
use bevy_proto_resource_tuples::*;

#[derive(Resource)]
struct ResourceA;

#[derive(Resource)]
struct ResourceB;

#[derive(Resource, Default)]
struct ResourceC;

#[derive(Resource, Default)]
struct ResourceD;
fn main() {
    App::new()
        .insert_resources((ResourceA, ResourceB))
        .init_resources::<(ResourceC, ResourceD)>()
        .run();
}
