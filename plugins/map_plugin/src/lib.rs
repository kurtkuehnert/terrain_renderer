use crate::bundles::MapBundle;
use crate::chunks::{
    handle_chunk_mesh_tasks, initialize_chunks, spawn_chunk_mesh_tasks, update_chunk_materials,
    update_visible_chunks, UPDATE_RATE,
};
use crate::map_data::{MapData, MaterialData};
use crate::map_pipeline::{add_map_graph, MapMaterial};

use bevy::core::FixedTimestep;
use bevy::prelude::*;
use bevy_inspector_egui::InspectableRegistry;

mod bundles;
mod chunks;
mod map_data;
mod map_generation;
mod map_pipeline;

/// A plugin that procedurally generates a map.
pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<MapMaterial>()
            .add_startup_system(setup)
            .add_system(initialize_chunks)
            .add_system(update_visible_chunks)
            .add_system(handle_chunk_mesh_tasks)
            .add_system_set(
                SystemSet::new()
                    .label("update map")
                    .with_system(update_chunk_materials)
                    .with_system(spawn_chunk_mesh_tasks)
                    .with_run_criteria(FixedTimestep::step(UPDATE_RATE)),
            );

        // add the render pipeline of the map to the graph
        add_map_graph(&mut app.world);

        // getting registry from world
        let mut registry = app
            .world
            .get_resource_or_insert_with(InspectableRegistry::default);

        // register components to be able to edit them in the inspector (works recursively)
        registry.register::<MapData>();
        registry.register::<MaterialData>();
    }
}

/// Creates the map.
fn setup(mut commands: Commands) {
    commands
        .spawn_bundle(MapBundle::default())
        .insert(Name::new("Map"));
}
