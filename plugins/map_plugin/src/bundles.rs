use crate::chunks::{Chunk, Map};
use crate::map_data::{MapData, MaterialData};
use crate::map_pipeline::{MapMaterial, MAP_PIPELINE_HANDLE};
use bevy::prelude::*;
use bevy::render::pipeline::RenderPipeline;
use bevy::render::render_graph::base::MainPass;

/// A bundle containing all the components required to spawn a chunk.
#[derive(Bundle)]
pub struct ChunkBundle {
    pub chunk: Chunk,
    pub name: Name,
    pub material: Handle<MapMaterial>,
    pub main_pass: MainPass,
    pub draw: Draw,
    pub visible: Visible,
    pub render_pipelines: RenderPipelines,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

impl Default for ChunkBundle {
    fn default() -> Self {
        Self {
            chunk: Chunk::default(),
            name: Name::from("Chunk"),
            material: Default::default(),
            main_pass: Default::default(),
            draw: Default::default(),
            visible: Visible::default(),
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                MAP_PIPELINE_HANDLE.typed(),
            )]),
            transform: Default::default(),
            global_transform: Default::default(),
        }
    }
}

impl ChunkBundle {
    pub fn new(chunk: Chunk, is_visible: bool) -> Self {
        ChunkBundle {
            name: Name::new(format!("Chunk ({},{})", chunk.coord.x, chunk.coord.y)),
            transform: Transform::from_xyz(chunk.position.x, 0.0, chunk.position.y),
            visible: Visible {
                is_visible,
                ..Default::default()
            },
            chunk,
            ..Default::default()
        }
    }
}

/// A bundle containing all the components required to spawn a map.
#[derive(Bundle, Default)]
pub struct MapBundle {
    pub map: Map,
    pub map_data: MapData,
    pub material_data: MaterialData,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}
