use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::{env, fs};

#[derive(Deserialize, Debug)]
struct TerrainEntry {
    name: String,
    size: u32,
    height: Option<f32>,
    urls: Option<Vec<String>>,
}

#[derive(Deserialize, Debug)]
struct TerrainSettings {
    terrain_dir: String,
    node_atlas_size: Option<u32>,
    preprocess: Option<bool>,
    terrain: String,
    terrains: Vec<TerrainEntry>,
}

pub struct Settings {
    pub terrain_path: String,
    pub node_atlas_size: u32,
    pub preprocess: bool,
    pub size: u32,
    pub height: f32,
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
            node_atlas_size: settings.node_atlas_size.unwrap_or(1028),
            preprocess: settings.preprocess.unwrap_or(false),
            size: entry.size,
            height: entry.height.unwrap_or(1250.0),
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
