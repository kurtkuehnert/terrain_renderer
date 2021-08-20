use crate::bundles::ChunkBundle;
use crate::map_data::MapData;
use crate::map_generation::{CHUNK_SIZE, HALF_CHUNK_SIZE};
use crate::map_pipeline::MapMaterial;
use bevy::prelude::*;
use bevy::render::camera::Camera;
use bevy::utils::HashMap;
use itertools::iproduct;

const MAX_VIEW_DIST: f32 = 300.0;
const CHUNK_VIEW_DIST: i32 = (MAX_VIEW_DIST / CHUNK_SIZE as f32 + 0.5) as i32;

/// Marks the entity as a chunk and stores its relative position to the map.
pub struct Chunk {
    position: Vec2, // center position of the chunk relative to the map
}

impl Chunk {
    fn new(position: Vec2) -> Chunk {
        Chunk { position }
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

/// Marks the entity as a map and stores the entity ids of all visible and loaded chunks.
#[derive(Default)]
pub struct Map {
    loaded_chunk_map: HashMap<IVec2, Entity>,
    visible_chunks: Vec<Entity>,
}

/// Updates the visibility of every chunk in the view distance and
/// spawns a new one if it is not yet loaded.
pub fn update_visible_chunks(
    mut commands: Commands,
    query_camera: Query<&Transform, (With<Camera>, Changed<Transform>)>,
    mut query_map: Query<(Entity, &mut Map, &Transform)>,
    mut query_chunk: Query<(&Chunk, &mut Visible)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<MapMaterial>>,
) {
    // fetch the camera position and end the system if there is none
    let camera_pos = match query_camera.single() {
        Ok(transform) => &transform.translation,
        Err(_) => return,
    };

    // update all chunks of the map
    for (map_entity, mut map, transform) in query_map.iter_mut() {
        let Map {
            ref mut visible_chunks,
            ref mut loaded_chunk_map,
        } = *map;

        let (relative_pos, chunk_x, chunk_y) =
            calculate_positions(camera_pos, &transform.translation);

        reset_visibility(&mut query_chunk, visible_chunks);

        // updates the visibility and loads all chunks in the visibility range of the camera
        for chunk_coord in iproduct!(
            -CHUNK_VIEW_DIST..=CHUNK_VIEW_DIST,
            -CHUNK_VIEW_DIST..=CHUNK_VIEW_DIST
        )
        .map(|(offset_x, offset_y)| IVec2::new(chunk_x + offset_x, chunk_y + offset_y))
        {
            // fetch or spawn the chunk at the coords
            loaded_chunk_map
                .entry(chunk_coord)
                .and_modify(|&mut chunk_entity| {
                    //  update the visibility of the current chunk
                    if let Ok((chunk, mut visible)) = query_chunk.get_mut(chunk_entity) {
                        if chunk.is_visible(relative_pos) {
                            visible.is_visible = true;
                            //  mark all visible chunks to reset visibility on next frame
                            visible_chunks.push(chunk_entity);
                        }
                    }
                })
                .or_insert_with(|| {
                    create_chunk(
                        map_entity,
                        chunk_coord,
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        visible_chunks,
                    )
                });
        }
    }
}

/// Creates a new chunk at the given coordinates and initializes its visibility.
fn create_chunk(
    map_entity: Entity,
    chunk_coord: IVec2,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<MapMaterial>>,
    visible_chunks: &mut Vec<Entity>,
) -> Entity {
    let map_data = MapData::default();
    let (mesh, material) = map_data.generate();
    let mesh = meshes.add(mesh);
    let material = materials.add(material);

    let position = Vec2::new(
        chunk_coord.x as f32 * CHUNK_SIZE as f32,
        chunk_coord.y as f32 * CHUNK_SIZE as f32,
    );
    let chunk = Chunk::new(position);
    // check visibility
    let is_visible = chunk.is_visible(position);

    // spawn the chunk
    let chunk_entity = commands
        .spawn_bundle(ChunkBundle {
            material,
            map_data,
            mesh,
            transform: Transform::from_xyz(position.x, 0.0, position.y),
            ..Default::default()
        })
        .insert(chunk)
        .insert(Visible {
            is_visible,
            ..Default::default()
        })
        .insert(Name::new(format!(
            "Chunk ({},{})",
            chunk_coord.x, chunk_coord.y
        )))
        .id();

    // make sure newly added chunks are set as visible
    if is_visible {
        visible_chunks.push(chunk_entity);
    }

    // add chunk as a child to the map
    commands.entity(map_entity).push_children(&[chunk_entity]);

    chunk_entity
}

/// Calculates the relative position of the camera and the chunk coordinates
fn calculate_positions(camera_pos: &Vec3, map_pos: &Vec3) -> (Vec2, i32, i32) {
    // calculate position of camera relative to the map
    let relative_pos = Vec2::new(camera_pos.x - map_pos.x, camera_pos.z - map_pos.z);

    // coordinates of the chunk the camera is in
    let chunk_x = (relative_pos.x / CHUNK_SIZE as f32).round() as i32;
    let chunk_y = (relative_pos.y / CHUNK_SIZE as f32).round() as i32;
    (relative_pos, chunk_x, chunk_y)
}

/// Resets visibility for all previously visible chunks.
fn reset_visibility(
    query_chunk: &mut Query<(&Chunk, &mut Visible)>,
    visible_chunks: &mut Vec<Entity>,
) {
    for chunk_entity in visible_chunks.iter() {
        if let Ok((_, mut visible)) = query_chunk.get_mut(*chunk_entity) {
            visible.is_visible = false;
        }
    }

    visible_chunks.clear();
}
