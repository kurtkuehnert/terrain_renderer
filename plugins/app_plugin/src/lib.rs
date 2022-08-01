#![allow(dead_code)]
#![allow(unused_imports)]

extern crate core;

mod camera;
mod parse;

use crate::camera::{set_camera_viewports, toggle_camera_system, SplitScreenCameras};
use bevy::pbr::{MaterialPipeline, MaterialPipelineKey};
use bevy::render::mesh::MeshVertexBufferLayout;
use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    reflect::TypeUuid,
    render::{camera::Projection, render_resource::*},
    window::PresentMode,
};
use bevy_terrain::{prelude::*, preprocess::prelude::*};
use smooth_bevy_cameras::{
    controllers::fps::{FpsCameraBundle, FpsCameraController, FpsCameraPlugin},
    LookTransformPlugin,
};
use std::time::Duration;

#[derive(AsBindGroup, TypeUuid, Clone)]
#[uuid = "003e1d5d-241c-45a6-8c25-731dee22d820"]
pub struct TerrainMaterial {}

impl Material for TerrainMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/terrain.wgsl".into()
    }
}

/// A plugin, which sets up the testing application.
pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WindowDescriptor {
            width: 1920.,
            height: 1080.,
            position: WindowPosition::At(Vec2::new(3600.0, 220.0)),
            title: "Terrain Rendering".into(),
            present_mode: PresentMode::Immediate,
            ..default()
        });

        app.add_plugins_with(DefaultPlugins, |plugins| {
            // plugins.disable::<bevy::log::LogPlugin>();
            // plugins.add_before::<bevy::asset::AssetPlugin, _>(bevy_web_asset::WebAssetPlugin);
            plugins
        })
        .add_plugin(LogDiagnosticsPlugin {
            debug: false,
            wait_duration: Duration::from_secs(5),
            filter: None,
        });
        //.add_plugin(FrameTimeDiagnosticsPlugin);

        // app.world
        //     .resource::<AssetServer>()
        //     .watch_for_changes()
        //     .unwrap();

        app.insert_resource(Msaa { samples: 4 })
            .init_resource::<SplitScreenCameras>()
            .add_plugin(LookTransformPlugin)
            .add_plugin(FpsCameraPlugin::default())
            .insert_resource(TerrainPipelineConfig {
                attachment_count: 2,
                ..default()
            })
            .add_plugin(TerrainPlugin)
            .add_plugin(TerrainMaterialPlugin::<TerrainMaterial>::default())
            .add_startup_system(setup_scene)
            .add_system(toggle_camera_system)
            .add_system(set_camera_viewports);
    }
}

fn sachsen(from_disk_loader: &mut AttachmentFromDiskLoader) -> TerrainConfig {
    // parse::parse(
    //     "data/dgm20_source",
    //     "data/dgm20_parsed",
    //     (278, 5780),
    //     "dgm20",
    //     parse::ParseFormat::XYZ {
    //         scale: 20,
    //         dimension: 100,
    //         max_height: 1000.0,
    //     },
    // );
    //
    // bevy_terrain::preprocess::preprocess_tiles(
    //     "data/dgm20_parsed",
    //     "assets/terrains/Sachsen/data/height",
    //     0,
    //     7,
    //     (0, 0),
    //     100,
    //     128,
    //     2,
    //     bevy_terrain::preprocess::ImageFormat::LUMA16,
    // );

    let mut config = TerrainConfig::new(16000, 128, 7, 300.0, 500, "terrains/Sachsen/".to_string());

    config.add_attachment_from_disk(from_disk_loader, "height", TextureFormat::R16Unorm, 128, 2);
    config.add_attachment_from_disk(from_disk_loader, "density", TextureFormat::R16Unorm, 128, 0);

    config
}

fn hartenstein_large(from_disk_loader: &mut AttachmentFromDiskLoader) -> TerrainConfig {
    // parse::parse(
    //     "data/dgm01_source",
    //     "data/dgm01_parsed",
    //     (328, 5620),
    //     "dgm01",
    //     parse::ParseFormat::XYZ {
    //         scale: 1,
    //         dimension: 2000,
    //         max_height: 1000.0,
    //     },
    // );
    // parse::parse(
    //     "data/dop20_source",
    //     "data/dop20_parsed",
    //     (328, 5620),
    //     "dop20",
    //     parse::ParseFormat::TIF,
    // );
    //
    // bevy_terrain::preprocess::preprocess_tiles(
    //     "data/dgm01_parsed",
    //     "assets/terrains/Hartenstein_large/data/height",
    //     0,
    //     7,
    //     (0, 0),
    //     2000,
    //     128,
    //     2,
    //     bevy_terrain::preprocess::ImageFormat::LUMA16,
    // );
    //
    // bevy_terrain::preprocess::preprocess_tiles(
    //     "data/dop20_parsed",
    //     "assets/terrains/Hartenstein_large/data/albedo",
    //     0,
    //     7,
    //     (0, 0),
    //     10000,
    //     128 * 5,
    //     1,
    //     bevy_terrain::preprocess::ImageFormat::RGBA,
    // );

    let mut config = TerrainConfig::new(
        16000,
        128,
        7,
        1000.0,
        500,
        "terrains/Hartenstein_large/".to_string(),
    );
    // let mut config = TerrainConfig::new(128, 7, 1000.0, "http://127.0.0.1:3535/".to_string());

    config.add_attachment_from_disk(from_disk_loader, "height", TextureFormat::R16Unorm, 128, 2);
    config.add_attachment_from_disk(from_disk_loader, "density", TextureFormat::R16Unorm, 128, 0);
    config.add_attachment_from_disk(
        from_disk_loader,
        "albedo",
        TextureFormat::Rgba8UnormSrgb,
        640,
        1,
    );

    config
}

fn hartenstein(from_disk_loader: &mut AttachmentFromDiskLoader) -> TerrainConfig {
    // bevy_terrain::preprocess::preprocess_tiles(
    //     "assets/terrains/Hartenstein/source/height.png",
    //     "assets/terrains/Hartenstein/data/height",
    //     0,
    //     5,
    //     (0, 0),
    //     4000,
    //     128,
    //     2,
    //     bevy_terrain::preprocess::ImageFormat::LUMA16,
    // );

    // preprocess_density(
    //     "assets/terrains/hartenstein/data/height",
    //     "assets/terrains/hartenstein/data/density",
    //     5,
    //     (0, 0),
    //     (32, 32),
    //     128,
    //     2,
    //     1000.0,
    // );

    // bevy_terrain::preprocess::preprocess_tiles(
    //     "assets/terrains/Hartenstein/source/albedo.png",
    //     "assets/terrains/Hartenstein/data/albedo",
    //     0,
    //     5,
    //     (0, 0),
    //     20000,
    //     128 * 5,
    //     1,
    //     bevy_terrain::preprocess::ImageFormat::RGBA,
    // );

    let mut config = TerrainConfig::new(
        4000,
        128,
        5,
        1000.0,
        500,
        "terrains/Hartenstein/".to_string(),
    );

    config.add_attachment_from_disk(from_disk_loader, "height", TextureFormat::R16Unorm, 128, 2);
    config.add_attachment_from_disk(from_disk_loader, "density", TextureFormat::R16Unorm, 128, 0);
    config.add_attachment_from_disk(
        from_disk_loader,
        "albedo",
        TextureFormat::Rgba8UnormSrgb,
        640,
        1,
    );

    config
}

fn witcher(from_disk_loader: &mut AttachmentFromDiskLoader) -> TerrainConfig {
    // bevy_terrain::preprocess::preprocess_tiles(
    //     "assets/terrains/Witcher/source/Witcher.png",
    //     "assets/terrains/Witcher/data/height",
    //     0,
    //     7,
    //     (0, 0),
    //     4096,
    //     128,
    //     2,
    //     bevy_terrain::preprocess::ImageFormat::LUMA16,
    // );

    // bevy_terrain::preprocess::preprocess_tiles(
    //     "assets/terrains/Witcher/source/Witcher2.png",
    //     "assets/terrains/Witcher/data/height",
    //     0,
    //     8,
    //     (0, 0),
    //     23552,
    //     128,
    //     2,
    //     bevy_terrain::preprocess::ImageFormat::LUMA16,
    // );

    // bevy_terrain::preprocess::preprocess_tiles(
    //     "assets/terrains/Witcher/source/Witcher3.png",
    //     "assets/terrains/Witcher/data/height",
    //     0,
    //     8,
    //     (0, 0),
    //     16384,
    //     128,
    //     2,
    //     bevy_terrain::preprocess::ImageFormat::LUMA16,
    // );

    // preprocess_density(
    //     "assets/terrains/Witcher/data/height",
    //     "assets/terrains/Witcher/data/density",
    //     7,
    //     (0, 0),
    //     (33, 33),
    //     128,
    //     2,
    //     250.0,
    // );

    let mut config = TerrainConfig::new(4096, 128, 8, 250.0, 500, "terrains/Witcher/".to_string());

    config.add_attachment_from_disk(from_disk_loader, "height", TextureFormat::R16Unorm, 128, 2);
    config.add_attachment_from_disk(from_disk_loader, "density", TextureFormat::R16Unorm, 128, 0);

    config
}

fn bevy(from_disk_loader: &mut AttachmentFromDiskLoader) -> TerrainConfig {
    // preprocess_tiles(
    //     "assets/terrains/Bevy/source/heightmap.png",
    //     "assets/terrains/Bevy/data/height",
    //     0,
    //     5,
    //     (0, 0),
    //     4096,
    //     128,
    //     2,
    //     ImageFormat::LUMA16,
    // );
    //
    // preprocess_density(
    //     "assets/terrains/Bevy/data/height",
    //     "assets/terrains/Bevy/data/density",
    //     5,
    //     (0, 0),
    //     (33, 33),
    //     128,
    //     2,
    //     500.0,
    // );

    let mut config = TerrainConfig::new(4096, 128, 5, 500.0, 1024, "terrains/Bevy/".to_string());

    config.add_attachment_from_disk(from_disk_loader, "height", TextureFormat::R16Unorm, 128, 2);
    config.add_attachment_from_disk(from_disk_loader, "density", TextureFormat::R16Unorm, 128, 0);

    config
}

fn setup_scene(
    mut commands: Commands,
    mut materials: ResMut<Assets<TerrainMaterial>>,
    mut cameras: ResMut<SplitScreenCameras>,
    mut quadtrees: ResMut<TerrainViewComponents<Quadtree>>,
    mut terrain_view_configs: ResMut<TerrainViewComponents<TerrainViewConfig>>,
) {
    let mut from_disk_loader = AttachmentFromDiskLoader::default();

    // let config = sachsen(&mut from_disk_loader);
    // let config = hartenstein_large(&mut from_disk_loader);
    // let config = hartenstein(&mut from_disk_loader);
    // let config = witcher(&mut from_disk_loader);
    let config = bevy(&mut from_disk_loader);

    let terrain = commands
        .spawn_bundle(TerrainBundle::new(config.clone()))
        .insert(from_disk_loader)
        .insert(materials.add(TerrainMaterial {}))
        .id();

    let view = commands
        .spawn_bundle(Camera3dBundle {
            projection: Projection::Perspective(PerspectiveProjection {
                far: 10000.0,
                ..default()
            }),
            ..default()
        })
        .insert_bundle(FpsCameraBundle::new(
            FpsCameraController::default(),
            Vec3::new(300.0, 1400.0, 300.0),
            Vec3::new(400.0, 1300.0, 400.0),
        ))
        .insert(TerrainView)
        .id();

    cameras.0.push(view);
    let view_config = TerrainViewConfig::new(&config, 13, 8.0, 6.0, 10.0, 0.2, 0.2, 0.2);
    let quadtree = Quadtree::from_configs(&config, &view_config);

    terrain_view_configs.insert((terrain, view), view_config);
    quadtrees.insert((terrain, view), quadtree);

    // let view2 = commands
    //     .spawn_bundle(Camera3dBundle {
    //         camera: Camera {
    //             priority: 1,
    //             ..default()
    //         },
    //         camera_3d: Camera3d {
    //             clear_color: ClearColorConfig::None,
    //             ..default()
    //         },
    //         projection: Projection::Perspective(PerspectiveProjection {
    //             far: 10000.0,
    //             ..default()
    //         }),
    //         transform: Transform::from_xyz(3000.0, 1500.0, 3000.0).looking_at(Vec3::ZERO, Vec3::Y),
    //         ..default()
    //     })
    //     .insert(TerrainView)
    //     .id();

    // cameras.0.push(view2);
    // let view_config = TerrainViewConfig::new(2.0, 8.0, 0.3);
    // let quadtree = Quadtree::new(&config, &view_config);
    // terrain_view_configs.insert((terrain, view2), view_config);
    // quadtrees.insert((terrain, view2), quadtree);

    commands.spawn_bundle(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::default(),
            illuminance: 10000.0,
            shadows_enabled: false,
            shadow_projection: Default::default(),
            shadow_depth_bias: 0.0,
            shadow_normal_bias: 0.0,
        },
        transform: Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4),
            ..default()
        },
        ..default()
    });
}
