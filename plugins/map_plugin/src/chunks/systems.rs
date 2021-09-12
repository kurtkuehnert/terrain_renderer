use crate::{
    bundles::ChunkBundle,
    chunks::{
        calculate_relativ_pos, start_chunk_task, update_visibility_and_mesh, Chunk, ChunkTask, Map,
        CHUNK_VIEW_DIST,
    },
    data::{LODData, MapData, MaterialData},
    generation::CHUNK_SIZE,
    pipeline::MapMaterial,
    unwrap_or,
};
use bevy::{
    math::{IVec2, Vec2},
    prelude::{
        Assets, BuildChildren, Changed, Children, Commands, Entity, Handle, Mesh, Parent, Query,
        Res, ResMut, Transform, Visible, With,
    },
    render::wireframe::Wireframe,
    tasks::AsyncComputeTaskPool,
};
use futures_lite::future;
use itertools::iproduct;

/// Interval in seconds, after which the maps are check for new map data and possibly updated.
pub const UPDATE_RATE: f64 = 0.1;

/// Marks the entity as a viwer of the map, around which chunks will be loaded.
pub struct Viewer;

/// Updates the visibility of every chunk in the view distance of any viewer
/// and spawns a new one if it is not yet loaded.
pub fn update_visible_chunks(
    mut commands: Commands,
    mut materials: ResMut<Assets<MapMaterial>>,
    thread_pool: Res<AsyncComputeTaskPool>,
    viewer_query: Query<&Transform, (With<Viewer>, Changed<Transform>)>,
    mut map_query: Query<(
        Entity,
        &mut Map,
        &Transform,
        &MapData,
        &MaterialData,
        &LODData,
    )>,
    mut chunk_query: Query<(&mut Chunk, &mut Visible)>,
) {
    // prevent visible chunks from beeing cleared if the viewer has not moved
    if viewer_query.is_empty() {
        return;
    }

    let (map_entity, mut map, map_transform, map_data, material_data, lod_data) =
        unwrap_or!(return, map_query.single_mut());

    let Map {
        ref mut visible_chunks,
        ref mut loaded_chunk_map,
    } = *map;

    // reset the visibility for all previously visible chunks
    for chunk_entity in visible_chunks.iter() {
        let (_, mut visible) = unwrap_or!(continue, chunk_query.get_mut(*chunk_entity));
        visible.is_visible = false;
    }

    visible_chunks.clear();

    // for each viewer update all visible chunks
    for relative_pos in viewer_query
        .iter()
        .map(|transform| calculate_relativ_pos(&transform.translation, map_transform))
    {
        // calculate the coordinates of the chunk
        let chunk_x = (relative_pos.x / CHUNK_SIZE as f32).round() as i32;
        let chunk_y = (relative_pos.y / CHUNK_SIZE as f32).round() as i32;

        // updates the visibility and loads all chunks in the visibility range of the camera
        for chunk_coord in iproduct!(
            -CHUNK_VIEW_DIST..=CHUNK_VIEW_DIST,
            -CHUNK_VIEW_DIST..=CHUNK_VIEW_DIST
        )
        .map(|(offset_x, offset_y)| IVec2::new(chunk_x + offset_x, chunk_y + offset_y))
        {
            // fetch or spawn the chunk at the coords and update its visibility
            loaded_chunk_map
                .entry(chunk_coord)
                // update the visibility of the current chunk
                .and_modify(|&mut chunk_entity| {
                    let (mut chunk, mut visible) =
                        unwrap_or!(return, chunk_query.get_mut(chunk_entity));

                    update_visibility_and_mesh(
                        &mut commands.entity(chunk_entity),
                        &thread_pool,
                        map_data,
                        lod_data,
                        visible_chunks,
                        relative_pos,
                        &mut chunk,
                        &mut visible,
                    )
                })
                // spawn a new chunk and initialize its visibility
                .or_insert_with(|| {
                    // spawn empty entity
                    let mut entity_commands = commands.spawn();
                    let chunk_entity = entity_commands.id();

                    // prepare components
                    let mut chunk = Chunk::new(chunk_coord);
                    let mut visible = Visible::default();
                    let material = materials.add(MapMaterial::new(material_data, map_data));

                    update_visibility_and_mesh(
                        &mut entity_commands,
                        &thread_pool,
                        map_data,
                        lod_data,
                        visible_chunks,
                        relative_pos,
                        &mut chunk,
                        &mut visible,
                    );

                    // insert all components into the new chunk entity
                    entity_commands.insert_bundle(ChunkBundle::new(chunk, visible, material));

                    // add chunk as a child to the map
                    commands.entity(map_entity).push_children(&[chunk_entity]);

                    chunk_entity
                });
        }
    }
}

/// Updates the materials of all chunks, if the material data of their map has changed.
pub fn update_materials_on_change(
    mut commands: Commands,
    mut materials: ResMut<Assets<MapMaterial>>,
    map_query: Query<(&Children, &MapData, &MaterialData), Changed<MaterialData>>,
    chunk_query: Query<(), With<Chunk>>,
) {
    let (children, map_data, material_data) = unwrap_or!(return, map_query.single());

    // update the materials of all chunks
    for &child in children.iter() {
        // validate that the child is a chunk
        unwrap_or!(continue, chunk_query.get(child));

        let mut entity_commands = commands.entity(child);

        // toggle the wireframe
        if material_data.wireframe {
            entity_commands.insert(Wireframe);
        } else {
            entity_commands.remove::<Wireframe>();
        }

        // create a new map material based on the updated material data of the map
        let material = materials.add(MapMaterial::new(material_data, map_data));

        // insert the new material into the chunk entity
        entity_commands.insert(material);
    }
}

/// Updates the meshes of all chunks asynchronously
/// by spawning a task that calculates the new meshes,
/// if the data of their map has changed.
pub fn update_mesh_on_change(
    mut commands: Commands,
    thread_pool: Res<AsyncComputeTaskPool>,
    map_query: Query<(&Children, &MapData), Changed<MapData>>,
    mut chunk_query: Query<&mut Chunk>,
) {
    let (children, map_data) = unwrap_or!(return, map_query.single());

    // spawns a task, which recalculates the mesh, for each chunk of a map if the map data has changed
    for &child in children.iter() {
        // validate that the child is a chunk
        let mut chunk = unwrap_or!(continue, chunk_query.get_mut(child));

        let entity_commands = &mut commands.entity(child);

        // remove the existing, outdated chunk task, if it exists
        entity_commands.remove::<ChunkTask>();

        // reset the chunk to clear all outdated meshes
        chunk.reset_chunk();

        start_chunk_task(entity_commands, &thread_pool, *map_data, &mut *chunk);
    }
}

/// Polls all chunk tasks and inserts their meshes into the chunk entity if the task is completed.
/// Additionally updates the material to represent the new map and water heights.
pub fn poll_chunk_tasks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<MapMaterial>>,
    map_query: Query<&MapData>,
    mut chunk_query: Query<(
        Entity,
        &Parent,
        &mut Chunk,
        &Handle<MapMaterial>,
        &mut ChunkTask,
    )>,
) {
    for (entity, parent, mut chunk, material, mut task) in chunk_query.iter_mut() {
        // poll, whether the task is ready yet
        if let Some((mesh, lod)) = future::block_on(future::poll_once(&mut *task)) {
            let mesh = meshes.add(mesh);

            chunk.insert_loaded_mesh(mesh.clone(), lod);

            // add new mesh and remove the completed task
            commands.entity(entity).insert(mesh).remove::<ChunkTask>();

            // update the map and water heights of the material with the new heights of the map
            if let Some(mut material) = materials.get_mut(material) {
                let map_data = unwrap_or!(continue, map_query.get(parent.0));

                material.map_height = map_data.map_height;
                material.water_height = map_data.get_water_height();
            }
        }
    }
}

/// Unloads all chunks outside of the max load distance.
pub fn unload_chunks(
    mut commands: Commands,
    viewer_query: Query<&Transform, With<Viewer>>,
    mut map_query: Query<(&mut Map, &Transform)>,
    chunk_query: Query<&Chunk>,
) {
    let (mut map, map_transform) = unwrap_or!(return, map_query.single_mut());

    let Map {
        visible_chunks: _,
        ref mut loaded_chunk_map,
    } = *map;

    let relative_positions: Vec<Vec2> = viewer_query
        .iter()
        .map(|transform| calculate_relativ_pos(&transform.translation, map_transform))
        .collect();

    // only retain chunks that should not be unloaded
    loaded_chunk_map.retain(|_, &mut chunk_entity| {
        let chunk = unwrap_or!(return false, chunk_query.get(chunk_entity));

        if relative_positions
            .iter()
            .all(|&pos| chunk.should_unload(pos))
        {
            // despawn entity with all of its components
            commands.entity(chunk_entity).despawn();
            return false;
        }

        true
    });
}
