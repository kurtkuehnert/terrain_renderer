use bevy::prelude::*;
use bevy_inspector_egui::{WorldInspectorParams, WorldInspectorPlugin};
use map_plugin::MapPlugin;

/// A plugin containing the entire game.
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(WorldInspectorPlugin::new())
            .add_plugin(MapPlugin)
            .insert_resource(WorldInspectorParams {
                despawnable_entities: true,
                ..Default::default()
            })
            .add_startup_system(setup);
    }
}

/// Creates the main camera for the game.
fn setup(mut commands: Commands) {
    // camera
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(-30.0, 5.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
}
