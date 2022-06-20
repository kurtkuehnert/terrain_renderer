use app_plugin::AppPlugin;
use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    window::PresentMode,
};
use std::time::Duration;

fn main() {
    let mut app = App::new();

    app.insert_resource(WindowDescriptor {
        width: 1920.,
        height: 1080.,
        position: Some(Vec2::new(3600.0, 220.0)),
        title: "Terrain Rendering".into(),
        present_mode: PresentMode::Immediate,
        ..default()
    })
    .add_plugins_with(DefaultPlugins, |plugins| {
        plugins.disable::<bevy::log::LogPlugin>();
        plugins
    })
    .add_plugin(LogDiagnosticsPlugin {
        debug: false,
        wait_duration: Duration::from_secs(5),
        filter: None,
    })
    .add_plugin(FrameTimeDiagnosticsPlugin);

    // app.world
    //     .resource::<AssetServer>()
    //     .watch_for_changes()
    //     .unwrap();

    app.add_plugin(AppPlugin);

    app.run()
}
