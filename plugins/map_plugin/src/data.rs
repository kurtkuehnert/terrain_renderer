use crate::generation::LOD_LEVELS;
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy_inspector_egui::{Inspectable, InspectableRegistry};

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
    #[inspectable(min = 0.0, max = 1000.0)]
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
            scale: 100.0,
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
    #[inspectable(collapse)]
    pub height_curve: HeightCurve,
    #[inspectable(collapse)]
    pub noise_data: NoiseData,
}

impl Default for MapData {
    fn default() -> Self {
        Self {
            map_height: 50.0,
            height_curve: Default::default(),
            noise_data: Default::default(),
        }
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

/// Stores the view distance for each level of detail.
#[derive(Inspectable, TypeUuid)]
#[uuid = "2e4c971f-1836-4fee-a628-03def3deb75d"]
pub struct LODData {
    pub lod_view_distance: [f32; LOD_LEVELS],
}

impl Default for LODData {
    fn default() -> Self {
        let mut lod_view_distance = [200.0; LOD_LEVELS];

        lod_view_distance
            .iter_mut()
            .enumerate()
            .for_each(|(i, distance)| *distance += i as f32 * 100.0);

        Self { lod_view_distance }
    }
}

/// Registers all types, that should be inspectable via the inspector plugin.
pub fn register_inspectable_types(world: &mut World) {
    let mut registry = world.get_resource_or_insert_with(InspectableRegistry::default);

    // register components to be able to edit them in the inspector (works recursively)
    registry.register::<MapData>();
    registry.register::<MaterialData>();
    registry.register::<LODData>();
}
