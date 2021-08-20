use crate::bundles::MapBundle;
use crate::chunks::update_visible_chunks;
use crate::map_data::MapData;
use crate::map_pipeline::{add_map_graph, MapMaterial};
use bevy::core::FixedTimestep;
use bevy::prelude::*;
use bevy::render::wireframe::Wireframe;
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
            .add_system(update_map.with_run_criteria(FixedTimestep::step(0.1)))
            .add_system(update_visible_chunks);

        // add the render pipeline of the map to the graph
        add_map_graph(&mut app.world);

        // getting registry from world
        let mut registry = app
            .world
            .get_resource_or_insert_with(InspectableRegistry::default);

        // register components to be able to edit them in the inspector (works recursively)
        registry.register::<MapData>();
    }
}

/// Creates the map.
fn setup(mut commands: Commands) {
    commands
        .spawn_bundle(MapBundle {
            transform: Transform::from_xyz(0.0, -20.0, 0.0),
            ..Default::default()
        })
        .insert(Name::new("Map"));
}

/// Updates the mesh and the material of the map, if the map data of it changed.
fn update_map(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<MapMaterial>>,
    query: Query<(Entity, &Handle<Mesh>, &Handle<MapMaterial>, &MapData), Changed<MapData>>,
) {
    // regenerate the mesh and the material of the map
    for (entity, mesh, material, map_data) in query.iter() {
        if map_data.wireframe {
            commands.entity(entity).insert(Wireframe);
        } else {
            commands.entity(entity).remove::<Wireframe>();
        }
        let mesh = meshes.get_mut(mesh).unwrap();
        let material = materials.get_mut(material).unwrap();
        let (new_mesh, new_material) = map_data.generate();
        *mesh = new_mesh;
        *material = new_material;
    }
}
