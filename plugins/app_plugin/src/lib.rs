mod camera;
mod parse;
mod parse_new;
mod terrain_setup;

use crate::camera::{setup_camera, toggle_camera_system};
use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    pbr::{
        wireframe::{Wireframe, WireframePlugin},
        Clusters, VisiblePointLights,
    },
    prelude::*,
    render::{primitives::Frustum, view::VisibleEntities},
    utils::HashSet,
};
use bevy_fly_camera::{FlyCamera, FlyCameraPlugin};
use bevy_terrain::{
    attachment_loader::TextureAttachmentFromDiskLoader, bundles::TerrainBundle,
    config::TerrainConfig, node_atlas::NodeAtlas, quadtree::Quadtree, TerrainPlugin,
};
use std::{any::TypeId, time::Duration};

/// A plugin, which sets up the testing application.
pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        // enable asset hot reloading
        app.world
            .resource::<AssetServer>()
            .watch_for_changes()
            .unwrap();

        let mut ignore_components: HashSet<TypeId> = HashSet::default();
        ignore_components.insert(TypeId::of::<Camera>());
        ignore_components.insert(TypeId::of::<GlobalTransform>());
        ignore_components.insert(TypeId::of::<VisibleEntities>());
        ignore_components.insert(TypeId::of::<Frustum>());
        ignore_components.insert(TypeId::of::<Clusters>());
        ignore_components.insert(TypeId::of::<VisiblePointLights>());
        ignore_components.insert(TypeId::of::<PreviousParent>());
        ignore_components.insert(TypeId::of::<FlyCamera>());

        ignore_components.insert(TypeId::of::<Quadtree>());
        ignore_components.insert(TypeId::of::<NodeAtlas>());

        app.insert_resource(Msaa { samples: 4 })
            // .insert_resource(WorldInspectorParams {
            //     despawnable_entities: true,
            //     ignore_components,
            //     ..default()
            // })
            // .add_plugin(WorldInspectorPlugin::new())
            .add_plugin(WireframePlugin)
            .add_plugin(LogDiagnosticsPlugin {
                debug: false,
                wait_duration: Duration::from_secs(5),
                filter: None,
            })
            .add_plugin(FrameTimeDiagnosticsPlugin)
            .add_plugin(FlyCameraPlugin)
            .add_plugin(TerrainPlugin)
            .add_startup_system(setup_scene)
            .add_startup_system(setup_camera)
            .add_system(toggle_camera_system)
            .add_system(toggle_wireframe_system);
    }
}

fn setup_scene(mut commands: Commands) {
    // let mut config = TerrainConfig::new(
    //     128,
    //     7,
    //     UVec2::new(2, 2),
    //     1.0,
    //     200.0,
    //     "terrains/Sachsen/".to_string(),
    // );
    //
    // parse_new::parse_dgm_01("data/dgm01_source", "data/dgm01_parsed");
    // parse_new::parse_dgm_20("data/dgm20_source", "data/dgm20_parsed");
    // parse_new::combine_dgm_20(
    //     "data/dgm20_parsed",
    //     "assets/terrains/Sachsen/source/DGM20.png",
    // );
    // bevy_terrain::preprocess::generate_node_textures(
    //     &config,
    //     "assets/terrains/Sachsen/source/DGM20.png",
    //     "assets/terrains/Sachsen/data/height",
    // );

    let mut config = TerrainConfig::new(
        128,
        5,
        UVec2::new(2, 2),
        1.0,
        1000.0,
        "terrains/Hartenstein/".to_string(),
    );

    // bevy_terrain::preprocess::generate_node_textures(
    //     &config,
    //     "assets/terrains/Hartenstein/source/height.png",
    //     "assets/terrains/Hartenstein/data/height",
    //     128,
    // );
    // bevy_terrain::preprocess::generate_albedo_textures(
    //     &config,
    //     "assets/terrains/Hartenstein/source/albedo.png",
    //     "assets/terrains/Hartenstein/data/albedo",
    //     128 * 5,
    // );

    let mut from_disk_loader = TextureAttachmentFromDiskLoader::default();
    terrain_setup::setup_default_sampler(&mut config, 2);
    terrain_setup::setup_height_texture(&mut config, &mut from_disk_loader, 3, 128 + 4);
    terrain_setup::setup_albedo_texture(&mut config, &mut from_disk_loader, 4, 128 * 5 + 2);

    commands
        .spawn_bundle(TerrainBundle::new(config))
        .insert(from_disk_loader)
        .insert(Wireframe);
}

fn toggle_wireframe_system(
    mut commands: Commands,
    input: Res<Input<KeyCode>>,
    terrain_query: Query<(Entity, Option<&Wireframe>), With<TerrainConfig>>,
) {
    if input.just_pressed(KeyCode::W) {
        for (entity, wireframe) in terrain_query.iter() {
            match wireframe {
                None => commands.entity(entity).insert(Wireframe),
                Some(_) => commands.entity(entity).remove::<Wireframe>(),
            };
        }
    }
}
