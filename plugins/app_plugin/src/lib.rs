#![allow(dead_code)]
#![allow(unused_imports)]

extern crate core;

mod camera;
mod parse;
mod terrains;

use crate::{
    camera::{toggle_camera, update_camera_viewports, SplitScreenCameras},
    terrains::*,
};
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
use bevy_atmosphere::prelude::*;
use bevy_terrain::prelude::*;
use dolly::prelude::*;
use std::time::{Duration, Instant};

const ATTACHMENT_COUNT: usize = 3;
const SPLIT: bool = false;

#[derive(AsBindGroup, TypeUuid, Clone)]
#[uuid = "003e1d5d-241c-45a6-8c25-731dee22d820"]
pub struct TerrainMaterial {}

impl Material for TerrainMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/terrain.wgsl".into()
    }
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
            present_mode: PresentMode::AutoVsync,
            ..default()
        })
        .add_plugins_with(DefaultPlugins, |plugins| {
            // plugins.disable::<bevy::log::LogPlugin>();
            // plugins.add_before::<bevy::asset::AssetPlugin, _>(bevy_web_asset::WebAssetPlugin);
            plugins
        })
        // .add_plugin(FrameTimeDiagnosticsPlugin)
        .add_plugin(LogDiagnosticsPlugin {
            debug: false,
            wait_duration: Duration::from_secs(5),
            filter: None,
        })
        .insert_resource(Msaa { samples: 4 })
        .init_resource::<SplitScreenCameras>()
        .insert_resource(TerrainPipelineConfig {
            attachment_count: ATTACHMENT_COUNT,
            ..default()
        })
        .insert_resource(Atmosphere {
            sun_intensity: 10.0,
            ..default()
        })
        .insert_resource(AtmosphereSettings { resolution: 64 })
        .add_plugin(AtmospherePlugin)
        .add_system(daylight_cycle)
        .add_plugin(TerrainPlugin)
        .add_plugin(TerrainDebugPlugin)
        .add_plugin(TerrainMaterialPlugin::<TerrainMaterial>::default())
        .add_startup_system(setup)
        .add_system(toggle_camera)
        .add_system(update_camera_viewports);
    }
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<TerrainMaterial>>,
    mut cameras: ResMut<SplitScreenCameras>,
    mut quadtrees: ResMut<TerrainViewComponents<Quadtree>>,
    mut terrain_view_configs: ResMut<TerrainViewComponents<TerrainViewConfig>>,
) {
    let mut preprocessor = Preprocessor::default();
    let mut loader = AttachmentFromDiskLoader::default();

    // let config = bevy(&mut preprocessor, &mut loader);
    // let config = witcher(&mut preprocessor, &mut loader, 3);
    // let config = sachsen_20(&mut preprocessor, &mut loader);
    let config = terrain(&mut preprocessor, &mut loader, "Hartenstein", 2);
    // let config = terrain(&mut preprocessor, &mut loader, "Hartenstein_large", 8);
    // let config = terrain(&mut preprocessor, &mut loader, "Sachsen", 113);

    // parse_dgm("Hartenstein_large");
    // parse_dom("Sachsen");
    // parse_dop("Hartenstein_large");
    // preprocess(&config, preprocessor);

    let terrain = commands
        .spawn((
            TerrainBundle::new(config.clone()),
            loader,
            materials.add(TerrainMaterial {}),
        ))
        .id();

    let view = commands
        .spawn((
            TerrainView,
            DebugCamera {
                rig: CameraRig::builder()
                    .with(Position::new(Vec3::new(-200.0, 1000.0, -200.0)))
                    .with(YawPitch {
                        yaw_degrees: -135.0,
                        pitch_degrees: 0.0,
                    })
                    .with(Smooth::new_position_rotation(3.0, 1.5))
                    .build(),
                // rig: CameraRig::builder()
                //     .with(Position::new(Vec3::new(30000.0, 10000.0, 30000.0)))
                //     .with(YawPitch {
                //         yaw_degrees: -135.0,
                //         pitch_degrees: 0.0,
                //     })
                //     .with(Smooth::new_position_rotation(4.0, 2.5))
                //     .build(),
                // active: false,
                // translation_speed: 10000.0,
                // rotation_speed: 5.0,
                // acceleration: 1.02,
                ..default()
            },
            Camera3dBundle {
                projection: Projection::Perspective(PerspectiveProjection {
                    far: 10000000.0,
                    ..default()
                }),
                ..default()
            },
            AtmosphereCamera(None),
        ))
        .id();

    cameras.0.push(view);

    let view_config = TerrainViewConfig {
        load_distance: 6.0,
        node_count: 12,
        tile_scale: 10.0,
        grid_size: 4,
        view_distance: 4.0 * config.leaf_node_size as f32,
        ..default()
    };
    let quadtree = Quadtree::from_configs(&config, &view_config);

    terrain_view_configs.insert((terrain, view), view_config);
    quadtrees.insert((terrain, view), quadtree);

    if SPLIT {
        let view2 = commands
            .spawn((
                TerrainView,
                DebugCamera::default(),
                Camera3dBundle {
                    camera: Camera {
                        priority: 1,
                        ..default()
                    },
                    camera_3d: Camera3d {
                        clear_color: ClearColorConfig::None,
                        ..default()
                    },
                    ..default()
                },
            ))
            .id();

        cameras.0.push(view2);
        let view_config = TerrainViewConfig {
            load_distance: 5.0,
            node_count: 8,
            tile_scale: 16.0,
            grid_size: 4,
            view_distance: 3.0 * config.leaf_node_size as f32,
            ..default()
        };
        let quadtree = Quadtree::from_configs(&config, &view_config);

        terrain_view_configs.insert((terrain, view2), view_config);
        quadtrees.insert((terrain, view2), quadtree);
    }

    commands.spawn((
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                illuminance: 15000.0,
                ..default()
            },
            transform: Transform::from_xyz(1.0, 1.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        Sun,
    ));

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 1000.0,
            ..default()
        },
        transform: Transform::from_xyz(0.0001, 1.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

#[derive(Component)]
struct Sun;

fn daylight_cycle(
    mut atmosphere: ResMut<Atmosphere>,
    mut query: Query<(&mut Transform, &mut DirectionalLight), With<Sun>>,
    time: Res<Time>,
) {
    let angle = time.time_since_startup().as_millis() as f32 / 1000.0 / 10.0;
    atmosphere.sun_position = Vec3::new(angle.cos(), angle.sin(), 0.0);

    if let Some((mut light_trans, mut directional)) = query.single_mut().into() {
        *light_trans =
            Transform::from_xyz(-angle.cos(), angle.sin(), 0.0).looking_at(Vec3::ZERO, Vec3::Y);
        directional.illuminance = angle.sin().max(0.0).powf(2.0) * 25000.0;
    }
}
