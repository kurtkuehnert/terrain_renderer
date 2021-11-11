use bevy::{prelude::*, reflect::TypeUuid};
use bevy_inspector_egui::{Inspectable, InspectableRegistry};

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

/// It is adjustable via the inspector.
#[derive(Inspectable, TypeUuid, Copy, Clone, Component)]
#[uuid = "fd016f46-f3a6-11eb-9a03-0242ac130003"]
pub struct MapTopologyData {
    pub map_size: Vec2,
    #[inspectable(min = 0.0, max = 100.0)]
    pub map_height: f32,
    #[inspectable(min = 0.0, max = 1.0, speed = 0.001)]
    pub water_level: f32,
    #[inspectable(collapse)]
    pub noise_data: NoiseData,
}

impl MapTopologyData {
    pub fn water_height(&self) -> f32 {
        self.map_height * self.water_level
    }
}

impl Default for MapTopologyData {
    fn default() -> Self {
        Self {
            map_size: Vec2::new(512.0, 512.0),
            map_height: 50.0,
            water_level: 0.3,
            noise_data: Default::default(),
        }
    }
}

/// It is adjustable via the inspector.
#[derive(Inspectable, TypeUuid, Component)]
#[uuid = "5de92f89-23f6-405e-8380-2ff1f1cec95b"]
pub struct MapAppearanceData {
    pub layer_colors: Vec<Color>,
    #[inspectable(min = 0.0, max = 1.0, speed = 0.01)]
    pub layer_heights: Vec<f32>,
    #[inspectable(min = 0.0, max = 1.0, speed = 0.01)]
    pub blend_values: Vec<f32>,
}

impl Default for MapAppearanceData {
    fn default() -> Self {
        Self {
            layer_colors: vec![
                Color::DARK_GRAY,
                Color::YELLOW,
                Color::GREEN,
                Color::DARK_GREEN,
                Color::DARK_GRAY,
                Color::WHITE,
            ],
            layer_heights: vec![0.2, 0.32, 0.37, 0.52, 0.8],
            blend_values: vec![0.2, 0.1, 0.1, 0.1, 0.2],
        }
    }
}

#[derive(Inspectable, TypeUuid, Copy, Clone, Component)]
#[uuid = "07862ce1-5cc7-42e2-83fa-f7c9a389e33f"]
pub struct ClipmapData {
    #[inspectable(min = 0, max = 10)]
    pub lod_levels: u32,
    #[inspectable(min = 0, max = 256)]
    pub side_length: u32,
    pub wireframe: bool,
}

impl Default for ClipmapData {
    fn default() -> Self {
        Self {
            lod_levels: 4,
            side_length: 64,
            wireframe: false,
        }
    }
}

/// Stores all parameters for the water materials.
/// It is adjustable via the inspector.
#[derive(Inspectable, TypeUuid, Copy, Clone, Component)]
#[uuid = "e53ae396-db66-4a9c-a3f1-6865964f7c10"]
pub struct WaterMaterialData {
    pub wave_sparsity: f32,
    #[inspectable(min = 0.0, max = 1.0, speed = 0.001)]
    pub wave_strength: f32,
    #[inspectable(min = 0.0, max = 1.0, speed = 0.01)]
    pub wave_speed: f32,
}

impl Default for WaterMaterialData {
    fn default() -> Self {
        Self {
            wave_sparsity: 100.0,
            wave_strength: 0.5,
            wave_speed: 0.15,
        }
    }
}

/// Registers all types, that should be inspectable via the inspector plugin.
pub fn register_inspectable_types(world: &mut World) {
    let mut registry: Mut<InspectableRegistry> =
        world.get_resource_or_insert_with(Default::default);

    // register components to be able to edit them in the inspector (works recursively)
    registry.register::<MapTopologyData>();
    registry.register::<MapAppearanceData>();
    registry.register::<ClipmapData>();
    registry.register::<WaterMaterialData>();
}
