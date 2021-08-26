use crate::bundles::ChunkBundle;
use crate::data::{LODData, MapData, MaterialData};
use crate::generation::{generate_chunk, CHUNK_SIZE, HALF_CHUNK_SIZE, LOD_LEVELS};
use crate::pipeline::MapMaterial;
use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use bevy::render::camera::Camera;
use bevy::render::wireframe::Wireframe;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use bevy::utils::HashMap;
use futures_lite::future;
use itertools::iproduct;

pub const UPDATE_RATE: f64 = 0.1;
const MAX_VIEW_DIST: f32 = 1000.0;
const CHUNK_VIEW_DIST: i32 = (MAX_VIEW_DIST / CHUNK_SIZE as f32 + 0.5) as i32;

/// Task that calculates the mesh of a chunk and its lod.
type ChunkTask = Task<(Mesh, usize)>;

/// The different states of visibility a chunk can be in.
#[derive(Eq, PartialEq)]
pub enum Visibility {
    /// Not visible.
    OutOfSight,
    /// Same lod as last update (visible).
    Unchanged,
    /// Different lod and mesh must be loaded (not yet visible).
    Load,
    /// Different lod and mesh is loaded (visible).
    Loaded(Handle<Mesh>),
}

/// Marks the entity as a map and stores the entity ids of all visible and loaded chunks.
#[derive(Default)]
pub struct Map {
    loaded_chunk_map: HashMap<IVec2, Entity>,
    visible_chunks: Vec<Entity>,
}

/// Marks the entity as a chunk and stores its relative position to the map,
/// as well as information about its level of detail.
#[derive(Default)]
pub struct Chunk {
    pub coord: IVec2,            // coordinates of the chunk inside the map
    pub position: Vec2,          // center position of the chunk relative to the map
    previous_lod: Option<usize>, // the last lod of the chunk
    // array storing the handles to already loaded lod meshes
    lod_meshes: [Option<Handle<Mesh>>; LOD_LEVELS],
}

impl Chunk {
    fn new(coord: IVec2) -> Chunk {
        Chunk {
            coord,
            position: Vec2::new(
                coord.x as f32 * CHUNK_SIZE as f32,
                coord.y as f32 * CHUNK_SIZE as f32,
            ),
            previous_lod: None,
            lod_meshes: Default::default(),
        }
    }

    //noinspection RsTypeCheck
    /// Updates and returns the visibility of the chunk.
    fn update_visibility(&mut self, camera_pos: Vec2, lod_data: &LODData) -> Visibility {
        let distance = self.distance_sqr(camera_pos);

        let mut current_lod = None;

        for (i, view_distance) in lod_data.lod_view_distance.iter().enumerate() {
            if distance < view_distance * view_distance {
                current_lod = Some(i);
                break;
            }
        }

        match current_lod {
            // not visible
            None => {
                self.previous_lod = None;
                Visibility::OutOfSight
            }
            // visible
            Some(lod) => {
                // check, whether the lod has changed
                if current_lod == self.previous_lod {
                    return Visibility::Unchanged;
                } else {
                    self.previous_lod = current_lod;
                }

                // check, whether the mesh is already loaded
                match &self.lod_meshes[lod] {
                    None => Visibility::Load,
                    Some(mesh) => Visibility::Loaded(mesh.clone()),
                }
            }
        }
    }

    /// Inserts a loaded mesh into the loaded lod meshes.
    fn insert_loaded_mesh(&mut self, mesh: Handle<Mesh>, lod: usize) {
        self.lod_meshes[lod] = Some(mesh);
    }

    /// Resets the chunk and clears all of its lod meshes.
    fn reset_chunk(&mut self) {
        self.lod_meshes = Default::default();
    }

    /// Calculates the squared distance of the point to the border of the chunk.
    fn distance_sqr(&self, point: Vec2) -> f32 {
        let dx = ((point.x - self.position.x).abs() - HALF_CHUNK_SIZE).max(0.0);
        let dy = ((point.y - self.position.y).abs() - HALF_CHUNK_SIZE).max(0.0);
        dx * dx + dy * dy
    }
}

/// Starts a task that updates the mesh of the chunk, by inserting it into the entity.
fn start_chunk_task(
    entity_commands: &mut EntityCommands,
    thread_pool: &AsyncComputeTaskPool,
    map_data: MapData,
    chunk: &Chunk,
) {
    let coord = chunk.coord;

    if let Some(lod) = chunk.previous_lod {
        // spawn the thread, which calculates the updated mesh of the map
        let task: ChunkTask =
            thread_pool.spawn(async move { (generate_chunk(&map_data, coord, lod), lod) });

        // add the task to the chunk entity
        entity_commands.insert(task);
    }
}

/// Updates the visibility of the chunk and loads the new mesh if the lod has changed.
#[allow(clippy::too_many_arguments)]
fn update_visibility(
    entity_commands: &mut EntityCommands,
    thread_pool: &AsyncComputeTaskPool,
    map_data: &MapData,
    lod_data: &LODData,
    visible_chunks: &mut Vec<Entity>,
    relative_pos: Vec2,
    chunk: &mut Chunk,
    visible: &mut Visible,
) {
    let visibility = chunk.update_visibility(relative_pos, lod_data);

    if visibility != Visibility::OutOfSight {
        visible.is_visible = true;
        visible_chunks.push(entity_commands.id());
    }

    // load the mesh if the lod has changed
    match visibility {
        Visibility::Loaded(mesh) => {
            entity_commands.insert(mesh);
        }
        Visibility::Load => {
            start_chunk_task(entity_commands, thread_pool, *map_data, chunk);
        }
        _ => {}
    }
}

/// Updates the visibility of every chunk in the view distance and
/// spawns a new one if it is not yet loaded.
pub fn update_visible_chunks(
    mut commands: Commands,
    thread_pool: Res<AsyncComputeTaskPool>,
    query_camera: Query<&Transform, (With<Camera>, Changed<Transform>)>,
    mut query_maps: Query<(Entity, &mut Map, &MapData, &LODData, &Transform)>,
    mut query_chunks: Query<(&mut Chunk, &mut Visible)>,
) {
    // fetch the camera position and end the system if there is none
    let camera_pos = match query_camera.single() {
        Ok(transform) => &transform.translation,
        Err(_) => return,
    };

    // update all chunks of the map
    for (map_entity, mut map, map_data, lod_data, transform) in query_maps.iter_mut() {
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
                // update the visibility of the current chunk
                .and_modify(|&mut chunk_entity| {
                    if let Ok((mut chunk, mut visible)) = query_chunks.get_mut(chunk_entity) {
                        update_visibility(
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

                    update_visibility(
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
                    entity_commands.insert_bundle(ChunkBundle::new(chunk, visible));

                    // add chunk as a child to the map
                    commands.entity(map_entity).push_children(&[chunk_entity]);

                    chunk_entity
                });
        }
    }
}

/// Initializes newly added chunks asynchronously by spawning a task
/// that calculates their meshes and creates their materials.
pub fn initialize_chunks(
    mut commands: Commands,
    mut materials: ResMut<Assets<MapMaterial>>,
    thread_pool: Res<AsyncComputeTaskPool>,
    query_maps: Query<(&MapData, &MaterialData)>,
    query_chunks: Query<(Entity, &Parent, &Chunk), Added<Chunk>>,
) {
    // for every newly added chunk initialize the material and spawn a task to calculate its mesh
    for (chunk_entity, map_entity, chunk) in query_chunks.iter() {
        if let Ok((map_data, material_data)) = query_maps.get(map_entity.0) {
            let mut entity_commands = commands.entity(chunk_entity);

            start_chunk_task(&mut entity_commands, &thread_pool, *map_data, chunk);

            // create a new map material based on the material data of the map
            let material = materials.add(MapMaterial::new(material_data, map_data.map_height));

            // add the task and the material to the chunk entity
            entity_commands.insert(material);

            if material_data.wireframe {
                entity_commands.insert(Wireframe);
            }
        }
    }
}

/// Updates the meshes of all chunks asynchronously
/// by spawning a task that calculates the new meshes,
/// if the data of their map has changed.
pub fn update_on_change(
    mut commands: Commands,
    thread_pool: Res<AsyncComputeTaskPool>,
    query_maps: Query<(&Children, &MapData), Changed<MapData>>,
    mut query_chunks: Query<&mut Chunk>,
) {
    // spawns a task, which recalculates the mesh, for each chunk of a map if the map data has changed
    for (chunks, map_data) in query_maps.iter() {
        for &chunk_entity in chunks.iter() {
            if let Ok(mut chunk) = query_chunks.get_mut(chunk_entity) {
                // reset the chunk to clear all outdated meshes
                chunk.reset_chunk();

                start_chunk_task(
                    &mut commands.entity(chunk_entity),
                    &thread_pool,
                    *map_data,
                    &*chunk,
                );
            }
        }
    }
}

/// Polls all chunk tasks and inserts their meshes into the chunk entity if the task is completed.
/// Additionally updates the material to represent the new map height.
pub fn handle_chunk_tasks(
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

            // add new mesh and remove the completed task
            commands
                .entity(entity)
                .insert(mesh.clone())
                .remove::<ChunkTask>();

            chunk.insert_loaded_mesh(mesh, lod);

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
