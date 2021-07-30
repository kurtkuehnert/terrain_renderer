mod map;
mod map_generation;
mod map_pipeline;

use crate::map::MapPlugin;
use bevy::prelude::*;
use bevy_config_cam::ConfigCam;

/// A plugin containing the entire game.
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(ConfigCam)
            .add_plugin(MapPlugin)
            .add_startup_system(setup.system());
    }
}

fn setup(mut commands: Commands) {
    // basic light
    commands.spawn_bundle(LightBundle {
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });
}
