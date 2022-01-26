mod camera;

use crate::camera::{setup_camera, toggle_camera_system};
use bevy::{
    pbr::{Clusters, VisiblePointLights},
    prelude::*,
    render::{primitives::Aabb, primitives::Frustum, view::VisibleEntities},
    utils::HashSet,
};
use bevy_fly_camera::{FlyCamera, FlyCameraPlugin};
use bevy_inspector_egui::{WorldInspectorParams, WorldInspectorPlugin};
use bevy_terrain::{
    bundles::TerrainBundle,
    quadtree::{NodeAtlas, Nodes, Quadtree, TreeUpdate},
    render::terrain_data::TerrainData,
    terrain::TerrainConfig,
    TerrainPlugin,
};
use std::any::TypeId;

/// A plugin, which sets up the testing application.
pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        // enable asset hot reloading
        app.world
            .get_resource::<AssetServer>()
            .unwrap()
            .watch_for_changes()
            .unwrap();

        let mut ignore_components: HashSet<TypeId> = HashSet::default();
        ignore_components.insert(TypeId::of::<Camera>());
        ignore_components.insert(TypeId::of::<GlobalTransform>());
        ignore_components.insert(TypeId::of::<VisibleEntities>());
        ignore_components.insert(TypeId::of::<Frustum>());
        ignore_components.insert(TypeId::of::<Clusters>());
        ignore_components.insert(TypeId::of::<VisiblePointLights>());
        ignore_components.insert(TypeId::of::<ComputedVisibility>());
        ignore_components.insert(TypeId::of::<Children>());
        ignore_components.insert(TypeId::of::<Parent>());
        ignore_components.insert(TypeId::of::<PreviousParent>());
        ignore_components.insert(TypeId::of::<FlyCamera>());
        ignore_components.insert(TypeId::of::<Aabb>());

        ignore_components.insert(TypeId::of::<Quadtree>());
        ignore_components.insert(TypeId::of::<TreeUpdate>());
        ignore_components.insert(TypeId::of::<Nodes>());
        ignore_components.insert(TypeId::of::<NodeAtlas>());
        ignore_components.insert(TypeId::of::<TerrainData>());

        app.insert_resource(Msaa { samples: 4 })
            .insert_resource(WorldInspectorParams {
                despawnable_entities: true,
                ignore_components,
                ..Default::default()
            })
            .add_plugin(FlyCameraPlugin)
            .add_plugin(WorldInspectorPlugin::new())
            .add_plugin(TerrainPlugin)
            .add_startup_system(setup_scene)
            .add_startup_system(setup_camera)
            .add_system(toggle_camera_system);
    }
}

fn setup_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut terrain_data: ResMut<Assets<TerrainData>>,
) {
    let config = TerrainConfig::new(128, 3, UVec2::new(2, 2));

    // bevy_terrain::preprocess::generate_node_textures(
    //     &config,
    //     "assets/heightmaps/heightmap.png",
    //     "assets/output/",
    // );

    let height_texture = asset_server.load("heightmaps/heightmap.png");

    commands.spawn_bundle(TerrainBundle::new(
        config.clone(),
        &mut meshes,
        &mut terrain_data,
        height_texture,
    ));
}
