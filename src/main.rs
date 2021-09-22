use bevy::{
    prelude::{App, IVec2, ResMut, WindowDescriptor, Windows},
    render::wireframe::WireframePlugin,
    wgpu::{WgpuFeature::NonFillPolygonMode, WgpuFeatures, WgpuOptions},
    DefaultPlugins,
};
use game_plugin::GamePlugin;

/// Builds and runs the entire game.
fn main() {
    env_logger::init();
    App::new()
        .insert_resource(WindowDescriptor {
            width: 1400.,
            height: 1000.,
            title: "Map Generation".into(),
            ..Default::default()
        })
        .insert_resource(WgpuOptions {
            features: WgpuFeatures {
                features: vec![NonFillPolygonMode],
            },
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(GamePlugin)
        .add_plugin(WireframePlugin)
        .add_startup_system(setup)
        .run();
}

/// Positions the window on startup.
fn setup(mut windows: ResMut<Windows>) {
    let window = windows.get_primary_mut().unwrap();
    window.set_position(IVec2::new(2000, 100));
}
