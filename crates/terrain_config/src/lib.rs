use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::{env, fs};

#[derive(Deserialize, Debug)]
struct TerrainEntry {
    name: String,
    size: u32,
    urls: Option<Vec<String>>,
}

#[derive(Deserialize, Debug)]
struct TerrainConfig {
    terrain_dir: String,
    node_atlas_size: Option<u32>,
    preprocess: Option<bool>,
    terrain: String,
    terrains: Vec<TerrainEntry>,
}

pub struct Config {
    pub terrain_path: String,
    pub node_atlas_size: u32,
    pub preprocess: bool,
    pub size: u32,
    pub urls: Vec<String>,
}

impl TryFrom<TerrainConfig> for Config {
    type Error = anyhow::Error;

    fn try_from(config: TerrainConfig) -> Result<Self> {
        let entry = config
            .terrains
            .into_iter()
            .find(|entry| entry.name == config.terrain)
            .ok_or(anyhow!(
                "Could not find the terrain config named {}.",
                config.terrain
            ))?;

        Ok(Self {
            terrain_path: format!("{}/{}", config.terrain_dir, entry.name),
            node_atlas_size: config.node_atlas_size.unwrap_or(1028),
            preprocess: config.preprocess.unwrap_or(false),
            size: entry.size,
            urls: entry.urls.unwrap_or_default(),
        })
    }
}

pub fn load_config() -> Result<Config> {
    let mut path = env::current_exe()?;
    path.pop();
    path.push("config.toml");
    let contents = fs::read_to_string(path)?;
    let config: TerrainConfig = toml::from_str(&contents)?;
    let config = config.try_into()?;

    Ok(config)
}
