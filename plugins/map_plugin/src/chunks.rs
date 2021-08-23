use crate::bundles::ChunkBundle;
use crate::map_data::{MapData, MaterialData};
use crate::map_generation::{CHUNK_SIZE, HALF_CHUNK_SIZE};
use crate::map_pipeline::MapMaterial;
use bevy::prelude::*;
use bevy::render::camera::Camera;
use bevy::render::wireframe::Wireframe;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use bevy::utils::HashMap;
use futures_lite::future;
use itertools::iproduct;

pub const UPDATE_RATE: f64 = 0.1;
const MAX_VIEW_DIST: f32 = 300.0;
const CHUNK_VIEW_DIST: i32 = (MAX_VIEW_DIST / CHUNK_SIZE as f32 + 0.5) as i32;

/// Marks the entity as a map and stores the entity ids of all visible and loaded chunks.
#[derive(Default)]
pub struct Map {
    loaded_chunk_map: HashMap<IVec2, Entity>,
    visible_chunks: Vec<Entity>,
}

/// Marks the entity as a chunk and stores its relative position to the map.
#[derive(Default)]
pub struct Chunk {
    pub coord: IVec2,   // coordinates of the chunk inside the map
    pub position: Vec2, // center position of the chunk relative to the map
}

impl Chunk {
    fn new(coord: IVec2) -> Chunk {
        Chunk {
            coord,
            position: Vec2::new(
                coord.x as f32 * CHUNK_SIZE as f32,
                coord.y as f32 * CHUNK_SIZE as f32,
            ),
        }
    }

    /// Returns whether or not the chunk is currently visible based on the distance to the camera.
    fn is_visible(&self, camera_pos: Vec2) -> bool {
        self.distance(camera_pos) < MAX_VIEW_DIST
    }

    /// Calculates the distance of the point to the border of the chunk.
    fn distance(&self, point: Vec2) -> f32 {
        let dx = ((point.x - self.position.x).abs() - HALF_CHUNK_SIZE).max(0.0);
        let dy = ((point.y - self.position.y).abs() - HALF_CHUNK_SIZE).max(0.0);
        (dx * dx + dy * dy).sqrt()
    }
}

/// Updates the visibility of every chunk in the view distance and
/// spawns a new one if it is not yet loaded.
pub fn update_visible_chunks(
    mut commands: Commands,
    query_camera: Query<&Transform, (With<Camera>, Changed<Transform>)>,
    mut query_maps: Query<(Entity, &mut Map, &Transform)>,
    mut query_chunks: Query<(&Chunk, &mut Visible)>,
) {
    // fetch the camera position and end the system if there is none
    let camera_pos = match query_camera.single() {
        Ok(transform) => &transform.translation,
        Err(_) => return,
    };

    // update all chunks of the map
    for (map_entity, mut map, transform) in query_maps.iter_mut() {
        let Map {
            ref mut visible_chunks,
            ref mut loaded_chunk_map,
        } = *map;

        // calculates the relative position of the camera
        let relative_pos = Vec2::new(
            camera_pos.x - transform.translation.x,
            camera_pos.z - transform.translation.z,
        );

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
                .and_modify(|&mut chunk_entity| {
                    // update the visibility of the current chunk
                    if let Ok((chunk, mut visible)) = query_chunks.get_mut(chunk_entity) {
                        if chunk.is_visible(relative_pos) {
                            visible.is_visible = true;
                            visible_chunks.push(chunk_entity);
                        }
                    }
                })
                .or_insert_with(|| {
                    // spawn a new chunk and initialize its visibility
                    let chunk = Chunk::new(chunk_coord);
                    let is_visible = chunk.is_visible(relative_pos);

                    // spawn chunk
                    let chunk_entity = commands
                        .spawn_bundle(ChunkBundle::new(chunk, is_visible))
                        .id();

                    // add chunk as a child to the map
                    commands.entity(map_entity).push_children(&[chunk_entity]);

                    // make sure newly added chunks are marked as visible in the map
                    visible_chunks.push(chunk_entity);

                    chunk_entity
                });
        }
    }
}

/// Initializes newly added chunks by spawning a task to calculate their meshes and
/// creating their materials.
pub fn initialize_chunks(
    mut commands: Commands,
    mut materials: ResMut<Assets<MapMaterial>>,
    thread_pool: Res<AsyncComputeTaskPool>,
    query_maps: Query<(&MapData, &MaterialData)>,
    query_chunks: Query<(Entity, &Parent, &Chunk), Added<Chunk>>,
) {
    // for every newly added chunk initialize the material and spawn a task to calculate its mesh
    for (chunk_entity, map_entity, chunk) in query_chunks.iter() {
        if let Ok((&map_data, material_data)) = query_maps.get(map_entity.0) {
            let coord = chunk.coord;

            // spawn the thread, which calculates the updated mesh of the map
            let task = thread_pool.spawn(async move { map_data.generate(coord) });

            // create a new map material based on the material data of the map
            let material = materials.add(MapMaterial::new(material_data, map_data.map_height));

            // add the task and the material to the chunk entity
            commands.entity(chunk_entity).insert(task).insert(material);
        }
    }
}

/// Updates the meshes of all chunks asynchronously
/// by spawning a task that calculates the new meshes,
/// if the data of their map has changed.
pub fn spawn_chunk_mesh_tasks(
    mut commands: Commands,
    thread_pool: Res<AsyncComputeTaskPool>,
    query_maps: Query<(&Children, &MapData), Changed<MapData>>,
    query_chunks: Query<&Chunk>,
) {
    // spawns a task, which recalculates the mesh, for each chunk of a map if the map data has changed
    for (chunks, &map_data) in query_maps.iter() {
        for &chunk_entity in chunks.iter() {
            if let Ok(chunk) = query_chunks.get(chunk_entity) {
                let coord = chunk.coord;

                // spawn the thread, which calculates the updated mesh of the map
                let task = thread_pool.spawn(async move { map_data.generate(coord) });

                // add the task to the chunk entity
                commands.entity(chunk_entity).insert(task);
            }
        }
    }
}

/// Polls all tasks that generate meshes for the chunks and
/// inserts them into the chunk entity if the task is completed.
/// Additionally updates the material to represent the new map height.
pub fn handle_chunk_mesh_tasks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<MapMaterial>>,
    mut query_chunks: Query<(Entity, &Parent, &Handle<MapMaterial>, &mut Task<Mesh>), With<Chunk>>,
    query_maps: Query<&MapData>,
) {
    for (entity, parent, material, mut task) in query_chunks.iter_mut() {
        // poll, whether the task is ready yet
        if let Some(mesh) = future::block_on(future::poll_once(&mut *task)) {
            // add new mesh and remove the complete task
            commands
                .entity(entity)
                .insert(meshes.add(mesh))
                .remove::<Task<Mesh>>();

            // update the map height of the material with the new height of the map
            if let Ok(map_data) = query_maps.get(parent.0) {
                if let Some(mut material) = materials.get_mut(material) {
                    material.map_height = map_data.map_height;
                }
            }
        }
    }
}

/// Updates the materials of all chunks, if the material data of their map has changed.
pub fn update_chunk_materials(
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
