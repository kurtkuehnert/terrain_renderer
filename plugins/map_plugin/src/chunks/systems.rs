use crate::bundles::ChunkBundle;
use crate::chunks::{calculate_relativ_pos, update_visibility_and_mesh, CHUNK_VIEW_DIST};
use crate::data::{LODData, MapData, MaterialData};
use crate::generation::CHUNK_SIZE;
use crate::pipeline::MapMaterial;
use bevy::prelude::*;
use bevy::render::camera::Camera;
use bevy::render::wireframe::Wireframe;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use futures_lite::future;
use itertools::iproduct;

use super::{start_chunk_task, Chunk, ChunkTask, Map};

/// Interval in seconds, after which the maps are check for new map data and possibly updated.
pub const UPDATE_RATE: f64 = 0.1;

/// Updates the visibility of every chunk in the view distance and
/// spawns a new one if it is not yet loaded.
pub fn update_visible_chunks(
    mut commands: Commands,
    thread_pool: Res<AsyncComputeTaskPool>,
    query_camera: Query<&Transform, (With<Camera>, Changed<Transform>)>,
    mut materials: ResMut<Assets<MapMaterial>>,
    mut query_maps: Query<(
        Entity,
        &mut Map,
        &MapData,
        &MaterialData,
        &LODData,
        &Transform,
    )>,
    mut query_chunks: Query<(&mut Chunk, &mut Visible)>,
) {
    // fetch the camera position and end the system if there is none
    let camera_pos = match query_camera.single() {
        Ok(transform) => &transform.translation,
        Err(_) => return,
    };

    // update all chunks of the map
    for (map_entity, mut map, map_data, material_data, lod_data, transform) in query_maps.iter_mut()
    {
        let Map {
            ref mut visible_chunks,
            ref mut loaded_chunk_map,
        } = *map;

        // calculates the relative position of the camera
        let relative_pos = calculate_relativ_pos(camera_pos, transform);

        // calculate the coordinates of the chunk
        let chunk_x = (relative_pos.x / CHUNK_SIZE as f32).round() as i32;
        let chunk_y = (relative_pos.y / CHUNK_SIZE as f32).round() as i32;

        // reset the visibility for all previously visible chunks
        for chunk_entity in visible_chunks.iter() {
            if let Ok((_, mut visible)) = query_chunks.get_mut(*chunk_entity) {
                visible.is_visible = false;
            }
        }

        visible_chunks.clear();

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
                    if let Ok((mut chunk, mut visible)) = query_chunks.get_mut(chunk_entity) {
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
                    }
                })
                // spawn a new chunk and initialize its visibility
                .or_insert_with(|| {
                    // spawn empty entity
                    let mut entity_commands = commands.spawn();
                    let chunk_entity = entity_commands.id();

                    // prepare components
                    let mut chunk = Chunk::new(chunk_coord);
                    let mut visible = Visible::default();
                    let material =
                        materials.add(MapMaterial::new(material_data, map_data.map_height));

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
    query_maps: Query<(&Children, &MapData, &MaterialData), Changed<MaterialData>>,
) {
    // update the materials of all chunks of all maps
    for (chunks, map_data, material_data) in query_maps.iter() {
        for &chunk_entity in chunks.iter() {
            let mut entity_commands = commands.entity(chunk_entity);

            // toggle the wireframe
            if material_data.wireframe {
                entity_commands.insert(Wireframe);
            } else {
                entity_commands.remove::<Wireframe>();
            }

            // create a new map material based on the updated material data of the map
            let material = materials.add(MapMaterial::new(material_data, map_data.map_height));

            // insert the new material into the chunk entity
            entity_commands.insert(material);
        }
    }
}

/// Updates the meshes of all chunks asynchronously
/// by spawning a task that calculates the new meshes,
/// if the data of their map has changed.
pub fn update_mesh_on_change(
    mut commands: Commands,
    thread_pool: Res<AsyncComputeTaskPool>,
    query_maps: Query<(&Children, &MapData), Changed<MapData>>,
    mut query_chunks: Query<(&mut Chunk, Option<&mut ChunkTask>)>,
) {
    // spawns a task, which recalculates the mesh, for each chunk of a map if the map data has changed
    for (chunks, map_data) in query_maps.iter() {
        for &chunk_entity in chunks.iter() {
            if let Ok((mut chunk, task)) = query_chunks.get_mut(chunk_entity) {
                // Todo: cancel pending task
                /*
                if let Some(&task) = task {
                    task.cancel(); // cancel the pending task and ignore its return value
                }
                 */

                // reset the chunk to clear all outdated meshes
                chunk.reset_chunk();

                start_chunk_task(
                    &mut commands.entity(chunk_entity),
                    &thread_pool,
                    *map_data,
                    &mut *chunk,
                );
            }
        }
    }
}

/// Polls all chunk tasks and inserts their meshes into the chunk entity if the task is completed.
/// Additionally updates the material to represent the new map height.
pub fn poll_chunk_tasks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<MapMaterial>>,
    mut query_chunks: Query<(
        Entity,
        &Parent,
        &mut Chunk,
        &Handle<MapMaterial>,
        &mut ChunkTask,
    )>,
    query_maps: Query<&MapData>,
) {
    for (entity, parent, mut chunk, material, mut task) in query_chunks.iter_mut() {
        // poll, whether the task is ready yet
        if let Some((mesh, lod)) = future::block_on(future::poll_once(&mut *task)) {
            let mesh = meshes.add(mesh);

            chunk.insert_loaded_mesh(mesh.clone(), lod);

            // add new mesh and remove the completed task
            commands.entity(entity).insert(mesh).remove::<ChunkTask>();

            // update the map height of the material with the new height of the map
            if let Ok(map_data) = query_maps.get(parent.0) {
                if let Some(mut material) = materials.get_mut(material) {
                    material.map_height = map_data.map_height;
                }
            }
        }
    }
}

/// Unloads all chunks outside of the max load distance.
pub fn unload_chunks(
    mut commands: Commands,
    query_camera: Query<&Transform, With<Camera>>,
    mut query_maps: Query<(&mut Map, &Transform)>,
    query_chunks: Query<&Chunk>,
) {
    // fetch the camera position and end the system if there is none
    let camera_pos = match query_camera.single() {
        Ok(transform) => &transform.translation,
        Err(_) => return,
    };

    // update all chunks of the map
    for (mut map, transform) in query_maps.iter_mut() {
        let Map {
            visible_chunks: _,
            ref mut loaded_chunk_map,
        } = *map;

        // calculates the relative position of the camera
        let relative_pos = calculate_relativ_pos(camera_pos, transform);

        // only retain chunks that should not be unloaded
        loaded_chunk_map.retain(|_, &mut chunk_entity| {
            if let Ok(chunk) = query_chunks.get(chunk_entity) {
                if chunk.should_unload(relative_pos) {
                    // despawn entity with all of its components
                    commands.entity(chunk_entity).despawn();
                    return false;
                }
            }
            true
        });
    }
}
