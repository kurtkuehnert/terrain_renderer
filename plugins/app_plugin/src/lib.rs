#![allow(dead_code)]
#![allow(unused_imports)]

extern crate core;

mod camera;
mod parse;
mod terrain_setup;

use crate::camera::{set_camera_viewports, setup_camera, toggle_camera_system, SplitScreenCameras};
use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    render::camera::Projection,
    window::PresentMode,
};
use bevy_fly_camera::{FlyCamera, FlyCameraPlugin};
use bevy_terrain::{
    attachment_loader::TextureAttachmentFromDiskLoader,
    bundles::TerrainBundle,
    quadtree::Quadtree,
    terrain::TerrainConfig,
    terrain_view::{TerrainView, TerrainViewComponents, TerrainViewConfig},
    TerrainPlugin,
};
use smooth_bevy_cameras::controllers::fps::{
    FpsCameraBundle, FpsCameraController, FpsCameraPlugin,
};
use smooth_bevy_cameras::LookTransformPlugin;
use std::time::Duration;

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
            .add_plugin(FlyCameraPlugin)
            .add_plugin(LookTransformPlugin)
            .add_plugin(FpsCameraPlugin::default())
            .add_plugin(TerrainPlugin)
            .add_startup_system(setup_scene)
            .add_startup_system(setup_camera)
            .add_system(toggle_camera_system)
            .add_system(set_camera_viewports);
    }
}

fn sachsen(from_disk_loader: &mut TextureAttachmentFromDiskLoader) -> TerrainConfig {
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

    let mut config = TerrainConfig::new(128, 7, 300.0, "terrains/Sachsen/".to_string());

    terrain_setup::setup_default_sampler(&mut config, 1);
    terrain_setup::setup_height_texture(&mut config, from_disk_loader, 2, 128 + 4);

    config
}

fn hartenstein_large(from_disk_loader: &mut TextureAttachmentFromDiskLoader) -> TerrainConfig {
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

    let mut config = TerrainConfig::new(128, 7, 1000.0, "terrains/Hartenstein_large/".to_string());
    // let mut config = TerrainConfig::new(128, 7, 1000.0, "http://127.0.0.1:3535/".to_string());

    terrain_setup::setup_default_sampler(&mut config, 1);
    terrain_setup::setup_height_texture(&mut config, from_disk_loader, 2, 128 + 4);
    terrain_setup::setup_albedo_texture(&mut config, from_disk_loader, 3, 128 * 5 + 2);

    config
}

fn hartenstein(from_disk_loader: &mut TextureAttachmentFromDiskLoader) -> TerrainConfig {
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
    //
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

    let mut config = TerrainConfig::new(128, 5, 1000.0, "terrains/Hartenstein/".to_string());

    terrain_setup::setup_default_sampler(&mut config, 1);
    terrain_setup::setup_height_texture(&mut config, from_disk_loader, 2, 128 + 4);
    terrain_setup::setup_albedo_texture(&mut config, from_disk_loader, 3, 128 * 5 + 2);

    config
}

fn witcher(from_disk_loader: &mut TextureAttachmentFromDiskLoader) -> TerrainConfig {
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

    let mut config = TerrainConfig::new(128, 8, 2500.0, "terrains/Witcher/".to_string());

    terrain_setup::setup_default_sampler(&mut config, 1);
    terrain_setup::setup_height_texture(&mut config, from_disk_loader, 2, 128 + 4);

    config
}

fn setup_scene(
    mut commands: Commands,
    mut cameras: ResMut<SplitScreenCameras>,
    mut quadtrees: ResMut<TerrainViewComponents<Quadtree>>,
    mut terrain_view_configs: ResMut<TerrainViewComponents<TerrainViewConfig>>,
) {
    let mut from_disk_loader = TextureAttachmentFromDiskLoader::default();

    // let config = sachsen(&mut from_disk_loader);
    // let config = hartenstein_large(&mut from_disk_loader);
    // let config = hartenstein(&mut from_disk_loader);
    let config = witcher(&mut from_disk_loader);

    let terrain = commands
        .spawn_bundle(TerrainBundle::new(config.clone()))
        .insert(from_disk_loader)
        .id();

    let perspective_projection = PerspectiveProjection {
        far: 10000.0,
        ..default()
    };

    // let view = commands
    //     .spawn_bundle(Camera3dBundle {
    //         camera: Camera::default(),
    //         projection: Projection::Perspective(perspective_projection.clone()),
    //         transform: Transform::from_xyz(300.0, 750.0, 300.0).looking_at(Vec3::ZERO, Vec3::Y),
    //         ..default()
    //     })
    //     .insert(TerrainView)
    //     .insert(FlyCamera {
    //         accel: 8.0,
    //         friction: 3.0,
    //         max_speed: 16.0,
    //         sensitivity: 30.0,
    //         key_forward: KeyCode::Up,
    //         key_backward: KeyCode::Down,
    //         key_left: KeyCode::Left,
    //         key_right: KeyCode::Right,
    //         key_up: KeyCode::PageUp,
    //         key_down: KeyCode::PageDown,
    //         enabled: true,
    //         ..default()
    //     })
    //     .id();

    let view = commands
        .spawn_bundle(Camera3dBundle {
            projection: Projection::Perspective(perspective_projection.clone()),
            ..default()
        })
        .insert_bundle(FpsCameraBundle::new(
            FpsCameraController {
                enabled: false,
                ..default()
            },
            Vec3::new(300.0, 700.0, 300.0),
            Vec3::new(400.0, 650.0, 400.0),
        ))
        .insert(TerrainView)
        .id();

    cameras.0.push(view);

    let view_config = TerrainViewConfig::new(10000, 3.0, 50.0, 0.5);
    let quadtree = Quadtree::new(&config, &view_config);

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
    //         projection: Projection::Perspective(perspective_projection.clone()),
    //         transform: Transform::from_xyz(6000.0, 1000.0, 6000.0).looking_at(Vec3::ZERO, Vec3::Y),
    //         ..default()
    //     })
    //     .insert(TerrainView)
    //     .id();
    //
    // let view_config = TerrainViewConfig::new(16000, 8, 2.0, 4.0, 0.25);
    //
    // cameras.0.push(view2);
    // let quadtree = Quadtree::new(&config, &view_config);
    // terrain_view_configs.insert((terrain, view2), view_config);
    // quadtrees.insert((terrain, view2), quadtree);

    // commands.spawn_bundle(DirectionalLightBundle {
    //     transform: Transform {
    //         translation: Vec3::new(0.0, 2.0, 0.0),
    //         rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4),
    //         ..default()
    //     },
    //     ..default()
    // });

    // commands.spawn_bundle(PointLightBundle {
    //     transform: Transform::from_xyz(0.0, 200.0, 0.0),
    //     ..default()
    // });

    // let mut from_disk_loader = TextureAttachmentFromDiskLoader::default();
    // let config = hartenstein(&mut from_disk_loader);
    //
    // commands
    //     .spawn_bundle(TerrainBundle::new(config))
    //     .insert(from_disk_loader);
}
