use app_plugin::AppPlugin;
use bevy::{prelude::*, window::PresentMode};

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            width: 1920.,
            height: 1080.,
            position: Some(Vec2::new(3600.0, 220.0)),
            title: "Terrain Rendering".into(),
            present_mode: PresentMode::Immediate,
            ..default()
        })
        .add_plugins_with(DefaultPlugins, |plugins| {
            plugins.disable::<bevy::log::LogPlugin>()
        })
        .add_plugin(AppPlugin)
        .run();
}
