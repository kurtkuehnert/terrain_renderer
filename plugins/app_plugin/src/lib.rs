extern crate core;

mod camera;
mod parse;
mod terrain_setup;

use crate::camera::{set_camera_viewports, setup_camera, toggle_camera_system};
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::window::PresentMode;
use bevy_fly_camera::FlyCameraPlugin;
use bevy_terrain::{
    attachment_loader::TextureAttachmentFromDiskLoader, bundles::TerrainBundle,
    config::TerrainConfig, TerrainPlugin,
};
use bevy_web_asset::WebAssetPlugin;
use std::time::Duration;

/// A plugin, which sets up the testing application.
pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WindowDescriptor {
            width: 1920.,
            height: 1080.,
            position: Some(Vec2::new(3600.0, 220.0)),
            title: "Terrain Rendering".into(),
            present_mode: PresentMode::Immediate,
            ..default()
        });

        app.add_plugins_with(DefaultPlugins, |plugins| {
            // plugins.disable::<bevy::log::LogPlugin>();
            plugins.add_before::<bevy::asset::AssetPlugin, _>(WebAssetPlugin);
            plugins
        })
        .add_plugin(LogDiagnosticsPlugin {
            debug: false,
            wait_duration: Duration::from_secs(5),
            filter: None,
        })
        .add_plugin(FrameTimeDiagnosticsPlugin);

        // app.world
        //     .resource::<AssetServer>()
        //     .watch_for_changes()
        //     .unwrap();
        app.insert_resource(Msaa { samples: 4 })
            .add_plugin(FlyCameraPlugin)
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

    terrain_setup::setup_default_sampler(&mut config, 2);
    terrain_setup::setup_height_texture(&mut config, from_disk_loader, 3, 128 + 4);

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

    terrain_setup::setup_default_sampler(&mut config, 2);
    terrain_setup::setup_height_texture(&mut config, from_disk_loader, 3, 128 + 4);
    // terrain_setup::setup_albedo_texture(&mut config, from_disk_loader, 4, 128 * 5 + 2);

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

    terrain_setup::setup_default_sampler(&mut config, 2);
    terrain_setup::setup_height_texture(&mut config, from_disk_loader, 3, 128 + 4);
    terrain_setup::setup_albedo_texture(&mut config, from_disk_loader, 4, 128 * 5 + 2);

    config
}

fn setup_scene(mut commands: Commands) {
    let mut from_disk_loader = TextureAttachmentFromDiskLoader::default();

    // let config = sachsen(&mut from_disk_loader);
    let config = hartenstein_large(&mut from_disk_loader);
    // let config = hartenstein(&mut from_disk_loader);

    commands
        .spawn_bundle(TerrainBundle::new(config))
        .insert(from_disk_loader);

    // let mut from_disk_loader = TextureAttachmentFromDiskLoader::default();
    // let config = hartenstein(&mut from_disk_loader);
    //
    // commands
    //     .spawn_bundle(TerrainBundle::new(config))
    //     .insert(from_disk_loader);
}
