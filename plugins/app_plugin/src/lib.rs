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
    // parse_new::parse_dgm20("data/dgm20_source", "data/dgm20_parsed");
    // parse_new::combine_dgm20_as_dgm16(
    //     "data/dgm20_parsed",
    //     "assets/terrains/Sachsen/source/DGM16.png",
    // );
    // bevy_terrain::preprocess::generate_node_textures(
    //     &config,
    //     "assets/terrains/Sachsen/source/DGM16.png",
    //     "assets/terrains/Sachsen/data/height",
    //     128,
    // );

    let mut config = TerrainConfig::new(
        128,
        7,
        UVec2::new(2, 2),
        1.0,
        200.0,
        "terrains/Sachsen/".to_string(),
    );

    terrain_setup::setup_default_sampler(&mut config, 2);
    terrain_setup::setup_height_texture(&mut config, from_disk_loader, 3, 128 + 4);

    config
}

fn hartenstein_large(from_disk_loader: &mut TextureAttachmentFromDiskLoader) -> TerrainConfig {
    // parse_new::parse_dgm01("data/dgm01_source", "data/dgm01_parsed");
    // parse_new::parse_dop20("data/dop20_source", "data/dop20_parsed");
    // parse_new::combine_dgm01(
    //     "data/dgm01_parsed",
    //     "assets/terrains/Hartenstein/source/DGM01.png",
    // );

    let mut config = TerrainConfig::new(
        128,
        7,
        UVec2::new(1, 1),
        1.0,
        1000.0,
        "terrains/Hartenstein_large/".to_string(),
    );

    terrain_setup::setup_default_sampler(&mut config, 2);
    terrain_setup::setup_height_texture(&mut config, from_disk_loader, 3, 128 + 4);
    terrain_setup::setup_albedo_texture(&mut config, from_disk_loader, 4, 128 * 5 + 2);

    config
}

fn hartenstein(from_disk_loader: &mut TextureAttachmentFromDiskLoader) -> TerrainConfig {
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

    let mut config = TerrainConfig::new(
        128,
        5,
        UVec2::new(2, 2),
        1.0,
        1000.0,
        "terrains/Hartenstein/".to_string(),
    );

    terrain_setup::setup_default_sampler(&mut config, 2);
    terrain_setup::setup_height_texture(&mut config, from_disk_loader, 3, 128 + 4);
    terrain_setup::setup_albedo_texture(&mut config, from_disk_loader, 4, 128 * 5 + 2);

    config
}

fn setup_scene(mut commands: Commands) {
    // parse_new::parse_dgm20("data/dgm20_source", "data/dgm20_parsed");
    // parse_new::parse_dgm01("data/dgm01_source", "data/dgm01_parsed");
    // parse_new::parse_dop20("data/dop20_source", "data/dop20_parsed");

    // bevy_terrain::preprocess::new::preprocess_tiles(
    //     "data/dgm01_parsed",
    //     "assets/terrains/Test/data/height",
    //     0,
    //     7,
    //     (0, 0),
    //     2000,
    //     128,
    //     2,
    //     ImageFormat::LUMA16,
    // );

    // bevy_terrain::preprocess::new::preprocess_tiles(
    //     "data/dop20_parsed",
    //     "assets/terrains/Test/data/albedo",
    //     0,
    //     7,
    //     (0, 0),
    //     10000,
    //     128 * 5,
    //     1,
    //     ImageFormat::RGB,
    // );

    let mut from_disk_loader = TextureAttachmentFromDiskLoader::default();

    // let config = sachsen(&mut from_disk_loader);
    // let config = hartenstein_large(&mut from_disk_loader);
    let config = hartenstein(&mut from_disk_loader);

    commands
        .spawn_bundle(TerrainBundle::new(config))
        .insert(from_disk_loader);

    // let mut from_disk_loader = TextureAttachmentFromDiskLoader::default();
    // let config = hartenstein(&mut from_disk_loader);
    // commands
    //     .spawn_bundle(TerrainBundle::new(config))
    //     .insert(from_disk_loader);
}
