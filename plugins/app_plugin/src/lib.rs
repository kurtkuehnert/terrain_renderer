use bevy::prelude::*;

/// A plugin, which sets up the testing application.
pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        // enable asset hot reloading
        app.world
            .get_resource::<AssetServer>()
            .unwrap()
            .watch_for_changes()
            .unwrap();
        app.add_startup_system(setup_scene);
    }
}

/// Creates the map with the water.
fn setup_scene() {}
