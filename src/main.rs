use app_plugin::AppPlugin;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::{pbr::wireframe::WireframePlugin, prelude::*};
use std::time::Duration;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            width: 1400.,
            height: 1000.,
            position: Some(Vec2::new(3000.0, 100.0)),
            title: "Terrain Rendering".into(),
            vsync: false,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(AppPlugin)
        .add_plugin(WireframePlugin)
        .add_plugin(LogDiagnosticsPlugin {
            debug: false,
            wait_duration: Duration::from_secs(5),
            filter: None,
        })
        .add_plugin(FrameTimeDiagnosticsPlugin)
        .run();
}
