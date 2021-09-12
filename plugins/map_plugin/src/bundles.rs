use crate::{
    chunks::{Chunk, Map},
    data::{LODData, MapData, MaterialData},
    pipeline::{MapMaterial, MAP_PIPELINE_HANDLE},
    water::{
        pipeline::{WaterMaterial, WaterPass, WATER_PIPELINE_HANDLE},
        Water,
    },
};
use bevy::{
    core::Name,
    prelude::{Bundle, Draw, GlobalTransform, Handle, Mesh, RenderPipelines, Transform, Visible},
    render::{pipeline::RenderPipeline, render_graph::base::MainPass},
};

/// A bundle containing all the components required to spawn a chunk.
#[derive(Bundle)]
pub struct ChunkBundle {
    pub chunk: Chunk,
    pub name: Name,
    pub material: Handle<MapMaterial>,
    pub main_pass: MainPass,
    pub water_pass: WaterPass,
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
            water_pass: Default::default(),
            draw: Default::default(),
            visible: Default::default(),
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                MAP_PIPELINE_HANDLE.typed(),
            )]),
            transform: Default::default(),
            global_transform: Default::default(),
        }
    }
}

impl ChunkBundle {
    pub fn new(chunk: Chunk, visible: Visible, material: Handle<MapMaterial>) -> Self {
        Self {
            name: Name::new(format!("Chunk ({},{})", chunk.coord.x, chunk.coord.y)),
            transform: Transform::from_xyz(chunk.position.x, 0.0, chunk.position.y),
            chunk,
            visible,
            material,
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
    pub lod_data: LODData,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

/// A bundle containing all the components required to spawn the water.
#[derive(Bundle)]
pub struct WaterBundle {
    pub water: Water,
    pub mesh: Handle<Mesh>,
    pub material: Handle<WaterMaterial>,
    pub main_pass: MainPass,
    pub draw: Draw,
    pub visible: Visible,
    pub render_pipelines: RenderPipelines,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

impl Default for WaterBundle {
    fn default() -> Self {
        Self {
            water: Default::default(),
            mesh: Default::default(),
            material: Default::default(),
            main_pass: Default::default(),
            draw: Default::default(),
            visible: Default::default(),
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                WATER_PIPELINE_HANDLE.typed(),
            )]),
            transform: Default::default(),
            global_transform: Default::default(),
        }
    }
}
