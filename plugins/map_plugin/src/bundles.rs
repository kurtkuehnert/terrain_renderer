use crate::data::ClipmapData;
use crate::tile::Tile;
use crate::water::pipeline::WaterPass;
use crate::{
    data::{MapAppearanceData, MapTopologyData, WaterMaterialData},
    pipeline::{MapMaterial, MAP_PIPELINE_HANDLE},
    water::{
        pipeline::{WaterMaterial, WATER_PIPELINE_HANDLE},
        Water,
    },
};
use bevy::prelude::Children;
use bevy::{
    prelude::{Bundle, Draw, GlobalTransform, Handle, Mesh, RenderPipelines, Transform, Visible},
    render::{pipeline::RenderPipeline, render_graph::base::MainPass},
};

#[derive(Bundle)]
pub struct TileBundle {
    pub tile: Tile,
    pub mesh: Handle<Mesh>,
    pub map_material: Handle<MapMaterial>,
    pub main_pass: MainPass,
    pub water_pass: WaterPass,
    pub draw: Draw,
    pub visible: Visible,
    pub render_pipelines: RenderPipelines,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

impl Default for TileBundle {
    fn default() -> Self {
        Self {
            tile: Default::default(),
            mesh: Default::default(),
            map_material: Default::default(),
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

/// A bundle containing all the components required to spawn a map.
#[derive(Bundle)]
pub struct MapBundle {
    pub topology_data: MapTopologyData,
    pub appearance_data: MapAppearanceData,
    pub clipmap_data: ClipmapData,
    pub water_data: WaterMaterialData,
    pub map_material: Handle<MapMaterial>,
    pub children: Children,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

impl Default for MapBundle {
    fn default() -> Self {
        Self {
            topology_data: Default::default(),
            appearance_data: Default::default(),
            clipmap_data: Default::default(),
            water_data: Default::default(),
            map_material: Default::default(),
            children: Default::default(),
            transform: Default::default(),
            global_transform: Default::default(),
        }
    }
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
