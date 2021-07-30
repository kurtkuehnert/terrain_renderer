use crate::map_generation::MapShape;
use crate::map_pipeline::{MapMaterial, MapPipeline};
use bevy::core::FixedTimestep;
use bevy::prelude::*;
use bevy::render::pipeline::RenderPipeline;
use bevy_inspector_egui::{Inspectable, InspectorPlugin};

/// A tag component used to identify a map.
struct Map;

/// Stores all parameters of a map.
/// It is adjustable via the inspector.
#[derive(Inspectable)]
pub struct MapData {
    #[inspectable(min = 1, max = 1000)]
    pub width: usize,
    #[inspectable(min = 1, max = 1000)]
    pub height: usize,
    #[inspectable(min = 0.0, max = 100.0)]
    pub map_height: f32,
    #[inspectable(min = 0.0, max = 100.0)]
    pub scale: f64,
    pub seed: u64,
    #[inspectable(min = 1, max = 10)]
    pub octaves: u32,
    #[inspectable(min = 0.0, max = 1.0)]
    pub persistence: f32,
    #[inspectable(min = 1.0, max = 100.0)]
    pub lacunarity: f64,
    pub wireframe: bool,
}

impl Default for MapData {
    fn default() -> Self {
        MapData {
            width: 50,
            height: 50,
            map_height: 5.0,
            scale: 10.0,
            seed: 0,
            octaves: 4,
            persistence: 0.5,
            lacunarity: 3.0,
            wireframe: false,
        }
    }
}

impl MapData {
    pub fn new() -> Self {
        MapData::default()
    }

    /// Generates a mesh of the map with the parameters of the map data.
    fn as_mesh(&self) -> Mesh {
        MapShape::new(self).into()
    }
}

/// A plugin that procedurally generates a map.
pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(InspectorPlugin::<MapData>::new())
            .add_asset::<MapMaterial>()
            .init_resource::<MapPipeline>()
            .add_startup_system(setup.system())
            .add_system(
                update_map
                    .system()
                    .with_run_criteria(FixedTimestep::step(0.1)),
            );
    }
}

/// Creates the map and spawns an entity with its mesh.
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    map_data: Res<MapData>,
    map_pipeline: Res<MapPipeline>,
) {
    // prepare the map and the render pipeline of the mesh
    let mesh = meshes.add(map_data.as_mesh());
    let render_pipelines =
        RenderPipelines::from_pipelines(vec![RenderPipeline::new(map_pipeline.pipeline.clone())]);

    // create the map entity
    commands
        .spawn_bundle(PbrBundle {
            mesh,
            render_pipelines,
            transform: Transform::from_xyz(0.0, -10.0, 0.0),
            ..Default::default()
        })
        .insert(Map);
}

/// Updates the mesh of the map, if the map data of it changed.
fn update_map(
    map_data: Res<MapData>,
    mut meshes: ResMut<Assets<Mesh>>,
    query: Query<&Handle<Mesh>, With<Map>>,
) {
    // only update on change
    if !map_data.is_changed() {
        return;
    }

    // regenerate the mesh of the map
    for mesh in query.iter() {
        let mesh = meshes.get_mut(mesh.clone()).unwrap();
        *mesh = map_data.as_mesh();
    }
}
