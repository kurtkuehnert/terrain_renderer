mod camera;
mod parse;

use crate::camera::{setup_camera, toggle_camera_system};
use bevy::render::render_resource::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};
use bevy::utils::HashMap;
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
use bevy_terrain::attachments::{
    add_albedo_attachment_config, add_height_attachment_config, add_sampler_attachment_config,
    AttachmentFromDisk, AttachmentFromDiskConfig,
};
use bevy_terrain::{
    bundles::TerrainBundle, config::TerrainConfig, node_atlas::NodeAtlas, quadtree::Quadtree,
    TerrainPlugin,
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
    let mut config = TerrainConfig::new(128, 5, UVec2::new(2, 2), 1.0, 1000.0, 2048);

    add_sampler_attachment_config(&mut config);
    add_height_attachment_config(&mut config, 129);
    add_albedo_attachment_config(&mut config, 128 * 5);

    let mut a_config = HashMap::new();
    a_config.insert(
        "height_map".into(),
        AttachmentFromDisk {
            path: "output/height".into(),
            texture_descriptor: TextureDescriptor {
                label: None,
                size: Extent3d {
                    width: 129,
                    height: 129,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::R16Unorm,
                usage: TextureUsages::COPY_SRC
                    | TextureUsages::COPY_DST
                    | TextureUsages::TEXTURE_BINDING,
            },
        },
    );

    a_config.insert(
        "albedo_map".into(),
        AttachmentFromDisk {
            path: "output/albedo".into(),
            texture_descriptor: TextureDescriptor {
                label: None,
                size: Extent3d {
                    width: 128 * 5,
                    height: 128 * 5,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8UnormSrgb,
                usage: TextureUsages::COPY_SRC
                    | TextureUsages::COPY_DST
                    | TextureUsages::TEXTURE_BINDING,
            },
        },
    );

    // let path = "assets/heightmaps/Hartenstein.png";
    // parse::process_height(path, 2);
    // bevy_terrain::preprocess::generate_node_textures(&config, path, "assets/output/height");

    //let path = "assets/heightmaps/test.png";
    // parse::process_albedo(path, 2);
    //bevy_terrain::preprocess::generate_albedo_textures(&config, path, "assets/output/albedo");

    // let path = "assets/heightmaps/Hartenstein_albedo.png";
    // parse::process_albedo(path, 2);
    // bevy_terrain::preprocess::generate_albedo_textures(&config, path, "assets/output/albedo");

    // bevy_terrain::preprocess::generate_node_textures(
    //     &config,
    //     "assets/heightmaps/heightmap.png",
    //     "assets/output/",
    // );

    commands
        .spawn_bundle(TerrainBundle::new(config))
        .insert(Wireframe)
        .insert(AttachmentFromDiskConfig {
            attachments: a_config,
            handle_mapping: Default::default(),
        });
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
