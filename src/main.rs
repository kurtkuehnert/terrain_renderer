use app_plugin::AppPlugin;
use bevy::{pbr::wireframe::WireframePlugin, prelude::*};

/// Builds and runs the entire game.
fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            width: 1400.,
            height: 1000.,
            position: Some(Vec2::new(3000.0, 100.0)),
            title: "Terrain Rendering".into(),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(AppPlugin)
        .add_plugin(WireframePlugin)
        .run();
}
