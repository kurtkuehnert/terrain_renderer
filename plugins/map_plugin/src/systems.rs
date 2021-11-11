use crate::data::MapAppearanceData;
use crate::tile::Tile;
use crate::{
    data::{ClipmapData, MapTopologyData},
    generation::generate_height_map,
    pipeline::MapMaterial,
    tile::spawn_clipmap_tiles,
};
use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task},
};
use futures_lite::future;
use util::unwrap_or;

pub fn update_topology_on_change(
    mut commands: Commands,
    thread_pool: Res<AsyncComputeTaskPool>,
    map_query: Query<(Entity, &MapTopologyData), Changed<MapTopologyData>>,
) {
    let (map_entity, topology_data) = unwrap_or!(return, map_query.get_single());

    let topology_data = topology_data.clone();

    let task: TopologyTask = thread_pool.spawn(async move { generate_height_map(topology_data) });

    commands
        .entity(map_entity)
        .remove::<TopologyTask>()
        .insert(task);
}

type TopologyTask = Task<Texture>;

pub fn poll_topology_tasks(
    mut commands: Commands,
    mut materials: ResMut<Assets<MapMaterial>>,
    mut textures: ResMut<Assets<Texture>>,
    mut map_query: Query<(
        Entity,
        &MapTopologyData,
        &Handle<MapMaterial>,
        &mut TopologyTask,
    )>,
) {
    let (map_entity, topology_data, material, mut task) =
        unwrap_or!(return, map_query.get_single_mut());

    if let Some(height_map) = future::block_on(future::poll_once(&mut *task)) {
        materials
            .get_mut(material)
            .unwrap()
            .update_topology(textures.add(height_map), topology_data);

        commands.entity(map_entity).remove::<TopologyTask>();
    }
}

pub fn update_appearance_on_change(
    mut materials: ResMut<Assets<MapMaterial>>,
    map_query: Query<(&MapAppearanceData, &Handle<MapMaterial>), Changed<MapAppearanceData>>,
) {
    let (appearance_data, material) = unwrap_or!(return, map_query.get_single());

    materials
        .get_mut(material)
        .unwrap()
        .update_appearance(appearance_data);
}

pub fn update_clipmap(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut map_query: Query<
        (Entity, &Children, &ClipmapData, &Handle<MapMaterial>),
        Changed<ClipmapData>,
    >,
    tile_query: Query<(), With<Tile>>,
) {
    let (map_entity, children, clipmap_data, map_material) =
        unwrap_or!(return, map_query.get_single_mut());

    for &child in children.iter() {
        if tile_query.get(child).is_ok() {
            commands.entity(child).despawn_recursive();
        }
    }

    let tiles = spawn_clipmap_tiles(
        &mut commands,
        &mut meshes,
        clipmap_data,
        map_material.clone(),
    );

    commands.entity(map_entity).push_children(&tiles);
}
