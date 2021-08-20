use crate::chunks::Map;
use crate::map_data::MapData;
use crate::map_pipeline::{MapMaterial, MAP_PIPELINE_HANDLE};
use bevy::prelude::*;
use bevy::render::pipeline::RenderPipeline;
use bevy::render::render_graph::base::MainPass;

/// A bundle containing all the components required to spawn a chunk.
#[derive(Bundle)]
pub struct ChunkBundle {
    pub map_data: MapData,
    pub mesh: Handle<Mesh>,
    pub material: Handle<MapMaterial>,
    pub main_pass: MainPass,
    pub draw: Draw,
    pub render_pipelines: RenderPipelines,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

impl Default for ChunkBundle {
    fn default() -> Self {
        Self {
            map_data: Default::default(),
            mesh: Default::default(),
            material: Default::default(),
            main_pass: Default::default(),
            draw: Default::default(),
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                MAP_PIPELINE_HANDLE.typed(),
            )]),
            transform: Default::default(),
            global_transform: Default::default(),
        }
    }
}

/// A bundle containing all the components required to spawn a map.
#[derive(Bundle, Default)]
pub struct MapBundle {
    pub map: Map,
    pub map_data: MapData,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}
