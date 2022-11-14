use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::{env, fs};

#[derive(Deserialize, Debug)]
struct TerrainEntry {
    name: String,
    side_length: u32,
    height: Option<f32>,
    node_atlas_size: Option<u32>,
    texture_size: Option<u32>,
    border_size: Option<u32>,
    mip_level_count: Option<u32>,
    node_count: Option<u32>,
    load_distance: Option<f32>,
    view_distance: Option<f32>,
    urls: Option<Vec<String>>,
}

#[derive(Deserialize, Debug)]
struct TerrainSettings {
    terrain_dir: String,
    preprocess: Option<bool>,
    terrain: String,
    terrains: Vec<TerrainEntry>,
}

pub struct Settings {
    pub terrain_path: String,
    pub preprocess: bool,
    pub node_atlas_size: u32,
    pub height: f32,
    pub side_length: u32,
    pub texture_size: u32,
    pub border_size: u32,
    pub mip_level_count: u32,
    pub node_count: u32,
    pub load_distance: f32,
    pub view_distance: f32,
    pub urls: Vec<String>,
}

impl TryFrom<TerrainSettings> for Settings {
    type Error = anyhow::Error;

    fn try_from(settings: TerrainSettings) -> Result<Self> {
        let entry = settings
            .terrains
            .into_iter()
            .find(|entry| entry.name == settings.terrain)
            .ok_or(anyhow!(
                "Could not find the terrain config named {}.",
                settings.terrain
            ))?;

        Ok(Self {
            terrain_path: format!("{}/{}", settings.terrain_dir, entry.name),
            preprocess: settings.preprocess.unwrap_or(false),
            node_atlas_size: entry.node_atlas_size.unwrap_or(1028),
            side_length: entry.side_length,
            height: entry.height.unwrap_or(1250.0),
            texture_size: entry.texture_size.unwrap_or(512),
            border_size: entry.border_size.unwrap_or(1),
            mip_level_count: entry.mip_level_count.unwrap_or(1),
            node_count: entry.node_count.unwrap_or(12),
            load_distance: entry.load_distance.unwrap_or(6.0),
            view_distance: entry.view_distance.unwrap_or(4.0),
            urls: entry.urls.unwrap_or_default(),
        })
    }
}

pub fn load_settings() -> Result<Settings> {
    let mut path = env::current_exe()?;
    path.pop();
    path.push("config.toml");

    let mut contents = fs::read_to_string(path);

    if contents.is_err() {
        let mut path = env::current_dir()?;
        path.push("config.toml");

        contents = fs::read_to_string(path);
    }

    let contents = contents?;

    let settings: TerrainSettings = toml::from_str(&contents)?;
    let settings = settings.try_into()?;

    Ok(settings)
}
