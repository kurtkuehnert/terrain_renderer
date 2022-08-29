#![allow(dead_code)]
#![allow(unused_imports)]

extern crate core;

mod camera;
mod parse;

use crate::camera::{set_camera_viewports, toggle_camera_system, SplitScreenCameras};
use bevy::{
    asset::AssetServerSettings,
    core_pipeline::clear_color::ClearColorConfig,
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    reflect::TypeUuid,
    render::{camera::Projection, mesh::MeshVertexBufferLayout, render_resource::*},
    window::PresentMode,
};
use bevy_terrain::prelude::*;
use smooth_bevy_cameras::{
    controllers::fps::{FpsCameraBundle, FpsCameraController, FpsCameraPlugin},
    LookTransformPlugin,
};
use std::time::Duration;

const CHUNK_SIZE: u32 = 128;
const ATTACHMENT_COUNT: usize = 2;
const SPLIT: bool = false;

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
        // app.insert_resource(AssetServerSettings {
        //     watch_for_changes: true,
        //     ..default()
        // });
        app.insert_resource(WindowDescriptor {
            width: 1920.,
            height: 1080.,
            position: WindowPosition::At(Vec2::new(3600.0, 220.0)),
            title: "Terrain Rendering".into(),
            present_mode: PresentMode::Immediate,
            ..default()
        })
        .add_plugins_with(DefaultPlugins, |plugins| {
            // plugins.disable::<bevy::log::LogPlugin>();
            // plugins.add_before::<bevy::asset::AssetPlugin, _>(bevy_web_asset::WebAssetPlugin);
            plugins
        })
        .add_plugin(LogDiagnosticsPlugin {
            debug: false,
            wait_duration: Duration::from_secs(5),
            filter: None,
        })
        //.add_plugin(FrameTimeDiagnosticsPlugin);
        .insert_resource(Msaa { samples: 4 })
        .init_resource::<SplitScreenCameras>()
        .add_plugin(LookTransformPlugin)
        .add_plugin(FpsCameraPlugin::default())
        .insert_resource(TerrainPipelineConfig {
            attachment_count: ATTACHMENT_COUNT,
            ..default()
        })
        .add_plugin(TerrainPlugin)
        .add_plugin(TerrainDebugPlugin)
        .add_plugin(TerrainMaterialPlugin::<TerrainMaterial>::default())
        .add_startup_system(setup_scene)
        .add_system(toggle_camera_system)
        .add_system(set_camera_viewports);
    }
}

fn sachsen(
    preprocessor: &mut Preprocessor,
    from_disk_loader: &mut AttachmentFromDiskLoader,
) -> TerrainConfig {
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

    let mut config =
        TerrainConfig::new(16000, 512, 14, 300.0, 1024, "terrains/Sachsen".to_string());

    config.add_base_attachment(
        preprocessor,
        from_disk_loader,
        512,
        TileConfig {
            path: "assets/terrains/Sachsen/source/DGM16.png".to_string(),
            lod: 0,
            offset: Default::default(),
            size: 16000,
        },
    );
    config
}

fn hartenstein_large(
    preprocessor: &mut Preprocessor,
    from_disk_loader: &mut AttachmentFromDiskLoader,
) -> TerrainConfig {
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

    let mut config = TerrainConfig::new(
        16000,
        128,
        7,
        1000.0,
        500,
        "terrains/Hartenstein_large".to_string(),
    );

    config.add_base_attachment(
        preprocessor,
        from_disk_loader,
        CHUNK_SIZE,
        TileConfig {
            path: "data/dgm01_parsed".to_string(),
            lod: 0,
            offset: Default::default(),
            size: 4000,
        },
    );
    config.add_attachment_from_disk(
        preprocessor,
        from_disk_loader,
        AttachmentConfig {
            name: "albedo".to_string(),
            center_size: 5 * CHUNK_SIZE,
            border_size: 1,
            format: AttachmentFormat::Rgba8,
        },
        TileConfig {
            path: "data/dop20_parsed".to_string(),
            lod: 0,
            offset: Default::default(),
            size: 20000,
        },
    );

    config
}

fn hartenstein(
    preprocessor: &mut Preprocessor,
    from_disk_loader: &mut AttachmentFromDiskLoader,
) -> TerrainConfig {
    let mut config = TerrainConfig::new(
        4000,
        CHUNK_SIZE,
        5,
        1000.0,
        1024,
        "terrains/Hartenstein".to_string(),
    );

    config.add_base_attachment(
        preprocessor,
        from_disk_loader,
        CHUNK_SIZE,
        TileConfig {
            path: "assets/terrains/Hartenstein/source/height.png".to_string(),
            lod: 0,
            offset: Default::default(),
            size: 4000,
        },
    );
    config.add_attachment_from_disk(
        preprocessor,
        from_disk_loader,
        AttachmentConfig {
            name: "albedo".to_string(),
            center_size: 5 * CHUNK_SIZE,
            border_size: 1,
            format: AttachmentFormat::Rgba8,
        },
        TileConfig {
            path: "assets/terrains/Hartenstein/source/albedo.png".to_string(),
            lod: 0,
            offset: Default::default(),
            size: 20000,
        },
    );

    config
}

fn witcher(
    preprocessor: &mut Preprocessor,
    from_disk_loader: &mut AttachmentFromDiskLoader,
) -> TerrainConfig {
    let mut config = TerrainConfig::new(
        16384,
        CHUNK_SIZE,
        8,
        1500.0,
        1500,
        "terrains/Witcher".to_string(),
    );

    config.add_base_attachment(
        preprocessor,
        from_disk_loader,
        CHUNK_SIZE,
        TileConfig {
            path: "assets/terrains/Witcher/source/Witcher3.png".to_string(),
            lod: 0,
            offset: Default::default(),
            size: 16384,
        },
    );

    config
}

fn bevy(
    preprocessor: &mut Preprocessor,
    from_disk_loader: &mut AttachmentFromDiskLoader,
) -> TerrainConfig {
    let mut config = TerrainConfig::new(
        4096,
        CHUNK_SIZE,
        12,
        500.0,
        1024,
        "terrains/Bevy".to_string(),
    );

    config.add_base_attachment(
        preprocessor,
        from_disk_loader,
        CHUNK_SIZE,
        TileConfig {
            path: "assets/terrains/Bevy/source/heightmap.png".to_string(),
            lod: 0,
            offset: Default::default(),
            size: 4096,
        },
    );

    config
}

fn setup_scene(
    mut commands: Commands,
    mut materials: ResMut<Assets<TerrainMaterial>>,
    mut cameras: ResMut<SplitScreenCameras>,
    mut quadtrees: ResMut<TerrainViewComponents<Quadtree>>,
    mut terrain_view_configs: ResMut<TerrainViewComponents<TerrainViewConfig>>,
) {
    let mut preprocessor = Preprocessor::default();
    let mut from_disk_loader = AttachmentFromDiskLoader::default();

    // let config = sachsen(&mut preprocessor, &mut from_disk_loader);
    // let config = hartenstein_large(&mut preprocessor, &mut from_disk_loader);
    // let config = hartenstein(&mut preprocessor, &mut from_disk_loader);
    // let config = witcher(&mut preprocessor, &mut from_disk_loader);
    let config = bevy(&mut preprocessor, &mut from_disk_loader);

    // preprocessor.preprocess(&config);

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
    let view_config = TerrainViewConfig::new(&config, 10, 5.0, 2.0, 10.0, 0.2, 0.2, 0.2);
    let quadtree = Quadtree::from_configs(&config, &view_config);

    terrain_view_configs.insert((terrain, view), view_config);
    quadtrees.insert((terrain, view), quadtree);

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

    if SPLIT {
        let pos = config.terrain_size as f32 / 2.0;
        let view2 = commands
            .spawn_bundle(Camera3dBundle {
                camera: Camera {
                    priority: 1,
                    ..default()
                },
                camera_3d: Camera3d {
                    clear_color: ClearColorConfig::None,
                    ..default()
                },
                projection: Projection::Perspective(PerspectiveProjection {
                    far: 10000.0,
                    ..default()
                }),
                ..default()
            })
            .insert_bundle(FpsCameraBundle::new(
                FpsCameraController::default(),
                Vec3::new(pos, 5000.0, pos),
                Vec3::new(pos - 1.0, 0.0, pos),
            ))
            .insert(TerrainView)
            .id();

        cameras.0.push(view2);
        let view_config = TerrainViewConfig::new(&config, 8, 5.0, 3.0, 16.0, 0.2, 0.2, 0.2);
        let quadtree = Quadtree::from_configs(&config, &view_config);

        terrain_view_configs.insert((terrain, view2), view_config);
        quadtrees.insert((terrain, view2), quadtree);
    }
}
