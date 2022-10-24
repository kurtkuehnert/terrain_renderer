use bevy_terrain::prelude::*;
use std::{env, time::Instant};
use terrain_config::load_config;

const HEIGHT: f32 = 1250.0;
const TEXTURE_SIZE: u32 = 512;
const MIP_LEVEL_COUNT: u32 = 6;
const TILE_SIZE: u32 = 2000;
const HEIGHT_FORMAT: FileFormat = FileFormat::DTM;
const ALBEDO_FORMAT: FileFormat = FileFormat::QOI;
const TERRAIN_DIR: &str = "/Volumes/EXTERN_SSD/Terrains";
const DATA_DIR: &str = "/Volumes/EXTERN_HDD/Terrain_Data";

pub(crate) fn preprocess(config: &TerrainConfig, preprocessor: Preprocessor) {
    let start = Instant::now();
    preprocessor.preprocess(config);
    let duration = start.elapsed();
    println!("Time elapsed during preprocessing is: {:?}.", duration);
}

pub(crate) fn bevy(
    preprocessor: &mut Preprocessor,
    loader: &mut AttachmentFromDiskLoader,
) -> TerrainConfig {
    let size = 4096;

    let mut config = TerrainConfig::new(size, 10, 500.0, 512, format!("{TERRAIN_DIR}/Bevy"));

    config.add_base_attachment_from_disk(
        preprocessor,
        loader,
        BaseConfig::new(TEXTURE_SIZE, MIP_LEVEL_COUNT),
        TileConfig {
            path: format!("{TERRAIN_DIR}/Bevy/source/height.png"),
            size,
            file_format: FileFormat::PNG,
        },
    );

    config
}

pub(crate) fn witcher(
    preprocessor: &mut Preprocessor,
    loader: &mut AttachmentFromDiskLoader,
    version: u32,
) -> TerrainConfig {
    let (size, height) = match version {
        1 => (4096, 200.0),
        2 => (23552, 1000.0),
        3 => (16384, 1500.0),
        _ => panic!("Version not available."),
    };

    let mut config = TerrainConfig::new(
        size,
        16,
        height,
        1000,
        format!("{TERRAIN_DIR}/Witcher/Witcher{version}"),
    );

    config.add_base_attachment_from_disk(
        preprocessor,
        loader,
        BaseConfig::new(TEXTURE_SIZE, MIP_LEVEL_COUNT),
        TileConfig {
            path: format!("{TERRAIN_DIR}/Witcher/source/Witcher{version}.png"),
            size,
            file_format: FileFormat::PNG,
        },
    );

    config
}

pub(crate) fn terrain() -> (TerrainConfig, AttachmentFromDiskLoader) {
    let config = load_config().unwrap();
    let path = config.terrain_path;

    let mut preprocessor = Preprocessor::default();
    let mut loader = AttachmentFromDiskLoader::default();

    let mut terrain_config = TerrainConfig::new(
        config.size * TILE_SIZE,
        16,
        HEIGHT,
        config.node_atlas_size,
        path.clone(),
    );
    terrain_config.leaf_node_size = TEXTURE_SIZE - (1 << MIP_LEVEL_COUNT);

    terrain_config.add_attachment_from_disk(
        &mut preprocessor,
        &mut loader,
        AttachmentConfig::new(
            "dgm".to_string(),
            TEXTURE_SIZE,
            MIP_LEVEL_COUNT,
            AttachmentFormat::R16,
        ),
        TileConfig {
            path: format!("{path}/source/dgm"),
            size: TILE_SIZE,
            file_format: FileFormat::DTM,
        },
    );
    terrain_config.add_attachment_from_disk(
        &mut preprocessor,
        &mut loader,
        AttachmentConfig::new(
            "dom".to_string(),
            TEXTURE_SIZE,
            MIP_LEVEL_COUNT,
            AttachmentFormat::R16,
        ),
        TileConfig {
            path: format!("{path}/source/dom"),
            size: TILE_SIZE,
            file_format: FileFormat::DTM,
        },
    );
    terrain_config.add_attachment_from_disk(
        &mut preprocessor,
        &mut loader,
        AttachmentConfig::new(
            "dop".to_string(),
            TEXTURE_SIZE,
            MIP_LEVEL_COUNT,
            AttachmentFormat::Rgb8,
        ),
        TileConfig {
            path: format!("{path}/source/dop"),
            size: TILE_SIZE,
            file_format: FileFormat::QOI,
        },
    );

    if config.preprocess {
        preprocess(&terrain_config, preprocessor);
    }

    (terrain_config, loader)
}
