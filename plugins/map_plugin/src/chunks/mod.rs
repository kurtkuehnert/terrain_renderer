use crate::{
    data::{LODData, MapData},
    generation::{generate_chunk, NoiseMap, CHUNK_SIZE, HALF_CHUNK_SIZE, LOD_LEVELS},
};
use bevy::{
    ecs::system::EntityCommands,
    math::{IVec2, Vec2, Vec3},
    prelude::{Entity, Handle, Mesh, Transform, Visible},
    tasks::{AsyncComputeTaskPool, Task},
    utils::HashMap,
};
use std::sync::{Arc, Mutex};

pub mod systems;

/// Distance after which the chunk is unloaded to free memory.
const MAX_LOAD_DIST: f32 = 2000.0;
/// Distance after which the chunk is definitely not visible anymore to save GPU resources.
const MAX_VIEW_DIST: f32 = CHUNK_SIZE as f32 * 6.0;
/// View distance converted into chunk coordinates.
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
    // stores the noise map for generating the meshes
    noise_map: Arc<Mutex<Option<NoiseMap>>>,
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
            noise_map: Arc::new(Mutex::new(None)),
        }
    }

    //noinspection RsTypeCheck
    /// Updates and returns the visibility of the chunk.
    fn update_visibility(&mut self, viewer_pos: Vec2, lod_data: &LODData) -> Visibility {
        let distance = self.distance_sqr(viewer_pos);

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

    /// Resets the chunk by clearing all of its lod meshes and unloading its noise map.
    fn reset_chunk(&mut self) {
        self.lod_meshes = Default::default();

        self.noise_map = Arc::new(Mutex::new(None));
    }

    /// Checks whether the chunk is outside of the max load distance and hence should onload.
    fn should_unload(&self, camera_pos: Vec2) -> bool {
        self.distance_sqr(camera_pos) > MAX_LOAD_DIST * MAX_LOAD_DIST
    }

    /// Calculates the squared distance of the point to the border of the chunk.
    fn distance_sqr(&self, point: Vec2) -> f32 {
        let dx = ((point.x - self.position.x).abs() - HALF_CHUNK_SIZE).max(0.0);
        let dy = ((point.y - self.position.y).abs() - HALF_CHUNK_SIZE).max(0.0);
        dx * dx + dy * dy
    }
}

/// Calculates the relativ position of the camera to the map.
fn calculate_relativ_pos(camera_pos: &Vec3, map_transform: &Transform) -> Vec2 {
    Vec2::new(
        (camera_pos.x - map_transform.translation.x) / map_transform.scale.x,
        (camera_pos.z - map_transform.translation.z) / map_transform.scale.z,
    )
}

/// Starts a task that updates the mesh of the chunk, by inserting it into the entity.
fn start_chunk_task(
    entity_commands: &mut EntityCommands,
    thread_pool: &AsyncComputeTaskPool,
    map_data: MapData,
    chunk: &mut Chunk,
) {
    let coord = chunk.coord;
    let noise_map = Arc::clone(&chunk.noise_map);

    if let Some(lod) = chunk.previous_lod {
        // spawn the thread, which calculates the updated mesh of the map
        let task: ChunkTask = thread_pool.spawn(async move {
            let noise_map = &mut *noise_map.lock().unwrap();

            (generate_chunk(&map_data, noise_map, coord, lod), lod)
        });

        // add the task to the chunk entity
        entity_commands.insert(task);
    }
}

/// Updates the visibility of the chunk and loads the new mesh if the lod has changed.
#[allow(clippy::too_many_arguments)]
fn update_visibility_and_mesh(
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
