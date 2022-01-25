mod camera;

use crate::camera::{setup_camera, toggle_camera_system};
use bevy::asset::LoadState;
use bevy::{
    pbr::{Clusters, VisiblePointLights},
    prelude::*,
    render::{primitives::Frustum, view::VisibleEntities},
    utils::HashSet,
};
use bevy_fly_camera::{FlyCamera, FlyCameraPlugin};
use bevy_inspector_egui::{WorldInspectorParams, WorldInspectorPlugin};
use bevy_terrain::quadtree::{
    traverse_quadtree, update_atlas, update_load_status, update_nodes, Quadtree,
};
use bevy_terrain::terrain::TerrainConfig;
use bevy_terrain::{bundles::TerrainBundle, material::TerrainMaterialPlugin};
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
        ignore_components.insert(TypeId::of::<Quadtree>());
        ignore_components.insert(TypeId::of::<Children>());
        ignore_components.insert(TypeId::of::<Parent>());
        ignore_components.insert(TypeId::of::<PreviousParent>());
        ignore_components.insert(TypeId::of::<FlyCamera>());

        app.insert_resource(Msaa { samples: 4 })
            .insert_resource(WorldInspectorParams {
                despawnable_entities: true,
                ignore_components,
                ..Default::default()
            })
            .add_plugin(FlyCameraPlugin)
            .add_plugin(WorldInspectorPlugin::new())
            .add_plugin(TerrainMaterialPlugin)
            .add_startup_system(setup_scene)
            .add_startup_system(setup_camera)
            .add_system(traverse_quadtree.before("update_nodes"))
            .add_system(update_nodes.label("update_nodes"))
            .add_system(update_atlas.after("update_nodes"))
            .add_system(update_load_status)
            // .add_system(update_quadtree_on_change.label("update"))
            // .add_system(update_view_distance_on_change.label("update"))
            // .add_system(traverse_quadtree.after("update"))
            .add_system(toggle_camera_system);
    }
}

fn setup_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    // mut meshes: ResMut<Assets<Mesh>>,
    // mut materials: ResMut<Assets<TerrainMaterial>>,
) {
    let config = TerrainConfig::new(128, 3, UVec2::new(2, 2));

    // bevy_terrain::preprocess::generate_node_textures(
    //     &config,
    //     "assets/heightmaps/heightmap.png",
    //     "assets/output/",
    // );

    let handle1: Handle<Image> = asset_server.load("heightmaps/heightmap.png");
    let handle2: Handle<Image> = asset_server.load("heightmaps/heightmap.png");

    let state: LoadState = asset_server.get_load_state(handle1.clone());

    println!("{:?}", state);

    println!("{:?}", handle1.id);
    println!("{:?}", handle2.id);

    assert_eq!(handle1, handle2);

    commands.spawn_bundle(TerrainBundle::new(&config));

    commands.spawn_bundle(DirectionalLightBundle {
        directional_light: Default::default(),
        ..Default::default()
    });
}
