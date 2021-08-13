mod map;
mod map_data;
mod map_generation;
mod map_pipeline;

use crate::map::MapPlugin;
use bevy::prelude::*;
use bevy_config_cam::ConfigCam;
use bevy_inspector_egui::{WorldInspectorParams, WorldInspectorPlugin};

/// A plugin containing the entire game.
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(WorldInspectorParams {
            despawnable_entities: true,
            ..Default::default()
        })
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(ConfigCam)
        .add_plugin(MapPlugin)
        .add_startup_system(setup.system());
    }
}

fn setup(mut _commands: Commands) {}
