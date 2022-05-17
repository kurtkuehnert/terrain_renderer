use app_plugin::AppPlugin;
use bevy::{prelude::*, window::PresentMode};

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            width: 1400.,
            height: 1000.,
            position: Some(Vec2::new(3600.0, 220.0)),
            title: "Terrain Rendering".into(),
            present_mode: PresentMode::Immediate,
            ..default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(AppPlugin)
        .run();
}
