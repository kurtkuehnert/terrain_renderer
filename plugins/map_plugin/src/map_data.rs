use crate::map_generation::ChunkShape;
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy_inspector_egui::Inspectable;

/// Stores the parameters for the height adjustment of the map.
/// It is adjustable via the inspector.
#[derive(Inspectable, TypeUuid, Copy, Clone)]
#[uuid = "abe9653e-ff3e-11eb-9a03-0242ac130003"]
pub struct HeightCurve {
    #[inspectable(min = 0.0, max = 1.0, speed = 0.01)]
    pub water_level: f32,
    #[inspectable(min = 1.0, max = 5.0, speed = 0.01)]
    pub slope: f32,
}

impl Default for HeightCurve {
    fn default() -> Self {
        Self {
            water_level: 0.25,
            slope: 1.5,
        }
    }
}

impl HeightCurve {
    /// Adjusts height values to flatten out the water and lower layers.
    pub fn evaluate(&self, input: f32) -> f32 {
        if input < self.water_level {
            0.0
        } else {
            f32::powf(
                (input - self.water_level) / (1.0 - self.water_level),
                self.slope,
            )
        }
    }
}
/// Stores all parameters for the noise map generation.
/// It is adjustable via the inspector.
#[derive(Inspectable, TypeUuid, Copy, Clone)]
#[uuid = "243f32e0-f3ad-11eb-9a03-0242ac130003"]
pub struct NoiseData {
    pub seed: u64,
    #[inspectable(min = 0.0, max = 100.0)]
    pub scale: f64,
    #[inspectable(min = 1, max = 6)]
    pub octaves: u32,
    #[inspectable(min = 0.0, max = 1.0, speed = 0.01)]
    pub persistence: f32,
    #[inspectable(min = 1.0, max = 10.0, speed = 0.01)]
    pub lacunarity: f64,
}

impl Default for NoiseData {
    fn default() -> Self {
        Self {
            seed: 0,
            scale: 40.0,
            octaves: 4,
            persistence: 0.5,
            lacunarity: 3.0,
        }
    }
}

/// Stores all parameters of a map.
/// It is adjustable via the inspector.
#[derive(Inspectable, TypeUuid, Copy, Clone)]
#[uuid = "fd016f46-f3a6-11eb-9a03-0242ac130003"]
pub struct MapData {
    #[inspectable(min = 0.0, max = 100.0)]
    pub map_height: f32,
    #[inspectable(min = 0, max = 6)]
    pub level_of_detail: usize,
    #[inspectable(collapse)]
    pub height_curve: HeightCurve,
    #[inspectable(collapse)]
    pub noise_data: NoiseData,
}

impl Default for MapData {
    fn default() -> Self {
        Self {
            map_height: 10.0,
            level_of_detail: 0,
            height_curve: Default::default(),
            noise_data: Default::default(),
        }
    }
}

impl MapData {
    pub fn new() -> Self {
        Self::default()
    }

    /// Generates a mesh of the map with the parameters of the map data.
    pub fn generate(&self, chunk_coord: IVec2) -> Mesh {
        ChunkShape::new(self, chunk_coord).into()
    }
}

/// Stores the parameters for the map materials.
/// It is adjustable via the inspector.
#[derive(Inspectable, TypeUuid)]
#[uuid = "5de92f89-23f6-405e-8380-2ff1f1cec95b"]
pub struct MaterialData {
    pub wireframe: bool,
    pub layer_colors: Vec<Color>,
    #[inspectable(min = 0.0, max = 1.0, speed = 0.01)]
    pub layer_heights: Vec<f32>,
    #[inspectable(min = 0.0, max = 1.0, speed = 0.01)]
    pub blend_values: Vec<f32>,
}

impl Default for MaterialData {
    fn default() -> Self {
        Self {
            wireframe: false,
            layer_colors: vec![
                Color::BLUE,
                Color::GREEN,
                Color::DARK_GREEN,
                Color::DARK_GRAY,
                Color::WHITE,
            ],
            layer_heights: vec![0.2, 0.35, 0.5, 0.8],
            blend_values: vec![0.05, 0.05, 0.1, 0.15],
        }
    }
}
