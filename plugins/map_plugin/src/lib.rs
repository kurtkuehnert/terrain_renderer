use crate::bundles::MapBundle;
use crate::chunks::systems::{
    poll_chunk_tasks, unload_chunks, update_materials_on_change, update_mesh_on_change,
    update_visible_chunks, UPDATE_RATE,
};
use crate::data::register_inspectable_types;
use crate::pipeline::{add_map_graph, MapMaterial};

use bevy::core::FixedTimestep;
use bevy::prelude::*;

mod bundles;
mod chunks;
mod data;
mod generation;
mod pipeline;

/// A plugin that procedurally generates a map.
pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<MapMaterial>()
            .add_startup_system(setup)
            .add_system(update_visible_chunks)
            .add_system(poll_chunk_tasks)
            .add_system_set(
                SystemSet::new()
                    .label("update map")
                    .with_system(update_materials_on_change)
                    .with_system(update_mesh_on_change)
                    .with_system(unload_chunks)
                    .with_run_criteria(FixedTimestep::step(UPDATE_RATE)),
            );

        // add the render pipeline of the map to the graph
        add_map_graph(&mut app.world);

        register_inspectable_types(&mut app.world);
    }
}

/// Creates the map.
fn setup(mut commands: Commands) {
    commands
        .spawn_bundle(MapBundle::default())
        .insert(Name::new("Map"));
}
