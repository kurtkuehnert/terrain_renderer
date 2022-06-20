extern crate core;

mod camera;
mod parse;
mod parse_new;
mod terrain_setup;

use crate::camera::{set_camera_viewports, setup_camera, toggle_camera_system};
use bevy::prelude::*;
use bevy_fly_camera::FlyCameraPlugin;
use bevy_terrain::{
    attachment_loader::TextureAttachmentFromDiskLoader, bundles::TerrainBundle,
    config::TerrainConfig, TerrainPlugin,
};

/// A plugin, which sets up the testing application.
pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
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

    let mut config = TerrainConfig::new(128, 7, 200.0, "terrains/Sachsen/".to_string());

    terrain_setup::setup_default_sampler(&mut config, 2);
    terrain_setup::setup_height_texture(&mut config, from_disk_loader, 3, 128 + 4);

    config
}

fn hartenstein_large(from_disk_loader: &mut TextureAttachmentFromDiskLoader) -> TerrainConfig {
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

    terrain_setup::setup_default_sampler(&mut config, 2);
    terrain_setup::setup_height_texture(&mut config, from_disk_loader, 3, 128 + 4);
    terrain_setup::setup_albedo_texture(&mut config, from_disk_loader, 4, 128 * 5 + 2);

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
