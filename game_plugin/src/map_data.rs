use crate::map_generation::MapShape;
use crate::map_pipeline::MapMaterial;
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy_inspector_egui::Inspectable;

/// Stores all parameters for the noise map generation.
/// It is adjustable via the inspector.
#[derive(Inspectable, TypeUuid)]
#[uuid = "243f32e0-f3ad-11eb-9a03-0242ac130003"]
pub struct NoiseData {
    pub seed: u64,
    #[inspectable(min = 1, max = 1000)]
    pub width: usize,
    #[inspectable(min = 1, max = 1000)]
    pub height: usize,
    #[inspectable(min = 0.0, max = 100.0)]
    pub scale: f64,
    #[inspectable(min = 1, max = 10)]
    pub octaves: u32,
    #[inspectable(min = 0.0, max = 1.0)]
    pub persistence: f32,
    #[inspectable(min = 1.0, max = 100.0)]
    pub lacunarity: f64,
}

impl Default for NoiseData {
    fn default() -> Self {
        Self {
            seed: 0,
            width: 50,
            height: 50,
            scale: 10.0,
            octaves: 4,
            persistence: 0.5,
            lacunarity: 3.0,
        }
    }
}

/// Stores all parameters of a map.
/// It is adjustable via the inspector.
#[derive(Inspectable, TypeUuid)]
#[uuid = "fd016f46-f3a6-11eb-9a03-0242ac130003"]
pub struct MapData {
    pub wireframe: bool,
    #[inspectable(min = 0.0, max = 100.0)]
    pub map_height: f32,
    pub noise_data: NoiseData,
    pub colors: Vec<Color>,
    #[inspectable(min = 0.0, max = 1.0, speed = 0.01)]
    pub heights: Vec<f32>,
}

impl Default for MapData {
    fn default() -> Self {
        Self {
            wireframe: false,
            map_height: 5.0,
            noise_data: NoiseData::default(),
            colors: vec![
                Color::BLUE,
                Color::GREEN,
                Color::DARK_GREEN,
                Color::GRAY,
                Color::WHITE,
            ],
            heights: vec![0.2, 0.35, 0.5, 0.8],
        }
    }
}

impl MapData {
    pub fn new() -> Self {
        Self::default()
    }

    /// Generates a mesh of the map with the parameters of the map data.
    pub fn generate(&self) -> (Mesh, MapMaterial) {
        (MapShape::new(self).into(), MapMaterial::new(self))
    }
}
