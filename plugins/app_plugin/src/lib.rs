mod camera;

use crate::camera::{setup_camera, toggle_camera_system};
use bevy::{
    pbr::{Clusters, VisiblePointLights},
    prelude::*,
    render::{primitives::Aabb, primitives::Frustum, view::VisibleEntities},
    utils::HashSet,
};
use bevy_fly_camera::{FlyCamera, FlyCameraPlugin};
use bevy_inspector_egui::{RegisterInspectable, WorldInspectorParams, WorldInspectorPlugin};
use bevy_terrain::compute::{TerrainAsset, TerrainPlugin};
use bevy_terrain::{
    bundles::TerrainBundle,
    debug::{debug, info, TerrainDebugInfo},
    material::{TerrainMaterial, TerrainMaterialPlugin},
    pipeline::TerrainData,
    quadtree::{
        traverse_quadtree, update_load_status, update_nodes, NodeAtlas, Nodes, Quadtree,
        TreeUpdate, ViewDistance,
    },
    terrain::TerrainConfig,
    tile::Tile,
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
            .add_plugin(TerrainMaterialPlugin)
            .add_plugin(TerrainPlugin)
            .register_inspectable::<ViewDistance>()
            .register_inspectable::<TerrainDebugInfo>()
            .add_startup_system(setup_scene)
            .add_startup_system(setup_camera)
            .add_system(traverse_quadtree.before("update_nodes"))
            .add_system(update_nodes.label("update_nodes"))
            .add_system(update_load_status)
            .add_system(debug.after("update_nodes"))
            .add_system(info.after("update_nodes"))
            .add_system(toggle_camera_system);
    }
}

fn setup_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<TerrainMaterial>>,
    mut terrain_assets: ResMut<Assets<TerrainAsset>>,
) {
    let config = TerrainConfig::new(128, 3, UVec2::new(2, 2));

    // bevy_terrain::preprocess::generate_node_textures(
    //     &config,
    //     "assets/heightmaps/heightmap.png",
    //     "assets/output/",
    // );

    let height_texture = asset_server.load("heightmaps/heightmap.png");

    let terrain = commands
        .spawn_bundle(TerrainBundle::new(config.clone()))
        .insert(meshes.add(Tile::new(8, true).to_mesh()))
        .insert(materials.add(TerrainMaterial {
            height_texture,
            height: 100.0,
        }))
        .insert(TerrainDebugInfo::default())
        .insert(terrain_assets.add(TerrainAsset { config }))
        .id();
}
