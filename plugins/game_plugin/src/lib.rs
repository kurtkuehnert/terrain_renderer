use bevy::prelude::{App, Plugin};
use bevy_inspector_egui::{WorldInspectorParams, WorldInspectorPlugin};
use map_plugin::MapPlugin;

/// A plugin containing the entire game.
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(WorldInspectorPlugin::new())
            .add_plugin(MapPlugin)
            .insert_resource(WorldInspectorParams {
                sort_components: true,
                despawnable_entities: true,
                ..Default::default()
            });
    }
}
