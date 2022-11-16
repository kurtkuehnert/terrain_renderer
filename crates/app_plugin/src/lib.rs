use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{camera::Projection, render_resource::*},
    window::PresentMode,
};
use bevy_atmosphere::prelude::*;
use bevy_terrain::{debug::DebugTerrain, prelude::*};
use std::{f32::consts::TAU, time::Instant};
use terrain_settings::load_settings;

const TERRAIN_SHADER: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 24380770943559);

#[derive(AsBindGroup, TypeUuid, Clone)]
#[uuid = "003e1d5d-241c-45a6-8c25-731dee22d820"]
pub struct TerrainMaterial {}

impl Material for TerrainMaterial {
    fn vertex_shader() -> ShaderRef {
        TERRAIN_SHADER.typed().into()
    }
    fn fragment_shader() -> ShaderRef {
        TERRAIN_SHADER.typed().into()
    }
}

/// A plugin, which sets up the testing application.
pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                width: 1920.,
                height: 1080.,
                // position: WindowPosition::At(Vec2::new(3600.0, 220.0)),
                title: "Saxony Terrain Renderer".into(),
                present_mode: PresentMode::AutoVsync,
                ..default()
            },
            ..default()
        }))
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(Atmosphere {
            sun_intensity: 10.0,
            ..default()
        })
        .insert_resource(AtmosphereSettings { resolution: 64 })
        .add_plugin(AtmospherePlugin)
        .add_plugin(TerrainPlugin {
            attachment_count: 3,
        })
        .add_plugin(TerrainDebugPlugin)
        .add_plugin(TerrainMaterialPlugin::<TerrainMaterial>::default())
        .add_startup_system(setup)
        .add_system(daylight_cycle)
        .add_system(toggle_camera_and_height_data);

        app.world.resource_mut::<Assets<_>>().set_untracked(
            TERRAIN_SHADER,
            Shader::from_wgsl(include_str!("terrain.wgsl")),
        );

        let mut debug = app.world.resource_mut::<DebugTerrain>();
        debug.albedo = true;
    }
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<TerrainMaterial>>,
    mut quadtrees: ResMut<TerrainViewComponents<Quadtree>>,
    mut terrain_view_configs: ResMut<TerrainViewComponents<TerrainViewConfig>>,
) {
    let settings = load_settings().unwrap();

    let mut preprocessor = Preprocessor::default();
    let mut loader = AttachmentFromDiskLoader::default();

    let mut config = TerrainConfig::new(
        settings.side_length * settings.tile_size,
        settings.lod_count,
        settings.height,
        settings.node_atlas_size,
        settings.terrain_path.clone(),
    );
    config.leaf_node_size = settings.texture_size - 2 * settings.border_size;
    config.add_attachment_from_disk(
        &mut preprocessor,
        &mut loader,
        AttachmentConfig::new(
            "dtm".to_string(),
            settings.texture_size,
            settings.border_size,
            settings.mip_level_count,
            AttachmentFormat::R16,
        ),
        TileConfig {
            path: format!("{}/source/dtm", &settings.terrain_path),
            size: settings.tile_size,
            file_format: FileFormat::DTM,
        },
    );
    if settings.enable_dsm {
        config.add_attachment_from_disk(
            &mut preprocessor,
            &mut loader,
            AttachmentConfig::new(
                "dsm".to_string(),
                settings.texture_size,
                settings.border_size,
                settings.mip_level_count,
                AttachmentFormat::R16,
            ),
            TileConfig {
                path: format!("{}/source/dsm", &settings.terrain_path),
                size: settings.tile_size,
                file_format: FileFormat::DTM,
            },
        );
    } else {
        // parse and load dtm twice
        // a little hacky I know, but it does the trick
        config.add_attachment_from_disk(
            &mut preprocessor,
            &mut loader,
            AttachmentConfig::new(
                "dsm".to_string(),
                settings.texture_size,
                settings.border_size,
                settings.mip_level_count,
                AttachmentFormat::R16,
            ),
            TileConfig {
                path: format!("{}/source/dtm", &settings.terrain_path),
                size: settings.tile_size,
                file_format: FileFormat::DTM,
            },
        );
    }

    config.add_attachment_from_disk(
        &mut preprocessor,
        &mut loader,
        AttachmentConfig::new(
            "dop".to_string(),
            settings.texture_size,
            settings.border_size,
            settings.mip_level_count,
            AttachmentFormat::Rgb8,
        ),
        TileConfig {
            path: format!("{}/source/dop", &settings.terrain_path),
            size: settings.tile_size,
            file_format: FileFormat::QOI,
        },
    );

    if settings.preprocess {
        println!("Started preprocessing the terrain data. This might take a while ...");
        let start = Instant::now();
        preprocessor.preprocess(&config);
        let duration = start.elapsed();
        println!("Time elapsed during preprocessing is: {:?}.", duration);
    }

    load_node_config(&mut config);

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
            // DebugCamera::new(Vec3::new(3950.0, 2850.0, 6550.0), -135.0, -40.0),
            DebugCamera::new(Vec3::new(0.0, 1500.0, 0.0), 225.0, -30.0),
            Camera3dBundle {
                projection: Projection::Perspective(PerspectiveProjection {
                    far: 10000000.0, // required by the atmosphere plugin
                    ..default()
                }),
                ..default()
            },
            AtmosphereCamera(None),
        ))
        .id();

    let view_config = TerrainViewConfig {
        node_count: settings.node_count,
        load_distance: settings.load_distance,
        view_distance: settings.view_distance,
        ..default()
    };
    let quadtree = Quadtree::from_configs(&config, &view_config);

    terrain_view_configs.insert((terrain, view), view_config);
    quadtrees.insert((terrain, view), quadtree);

    commands.spawn((
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                illuminance: 15000.0,
                ..default()
            },
            transform: Transform::from_xyz(0.0, 4.0, -1.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        Sun::default(),
    ));
    commands.insert_resource(AmbientLight {
        brightness: 0.2,
        ..default()
    });
}

#[derive(Component)]
struct Sun {
    rotating: bool,
    angle: f32,
    period_duration: f32,
    illuminance: f32,
}

impl Default for Sun {
    fn default() -> Self {
        Self {
            rotating: false,
            angle: 0.0,
            period_duration: 30.0,
            illuminance: 25000.0,
        }
    }
}

fn daylight_cycle(
    input: Res<Input<KeyCode>>,
    mut atmosphere: ResMut<Atmosphere>,
    mut query: Query<(&mut Transform, &mut DirectionalLight, &mut Sun)>,
    time: Res<Time>,
) {
    let (mut transform, mut light, mut sun) = query.single_mut();

    if input.just_pressed(KeyCode::Z) {
        sun.rotating = !sun.rotating;
        println!(
            "Toggled the sun rotation {}.",
            if sun.rotating { "on" } else { "off" }
        )
    }
    if input.just_pressed(KeyCode::X) && sun.period_duration > 1.0 {
        sun.period_duration -= 1.0;
        println!(
            "Decreased the sun`s period duration to {}s.",
            sun.period_duration
        )
    }
    if input.just_pressed(KeyCode::Q) {
        sun.period_duration += 1.0;
        println!(
            "Increased the sun`s period duration to {}s.",
            sun.period_duration
        )
    }

    if sun.rotating {
        sun.angle += TAU * time.delta_seconds() / sun.period_duration;
        atmosphere.sun_position = Vec3::new(sun.angle.cos(), sun.angle.sin(), 0.0);

        *transform = Transform::from_xyz(-sun.angle.cos(), sun.angle.sin(), 0.0)
            .looking_at(Vec3::ZERO, Vec3::Y);
        light.illuminance = sun.angle.sin().max(0.0).powf(2.0) * sun.illuminance;
    }
}

fn toggle_camera_and_height_data(
    input: Res<Input<KeyCode>>,
    mut camera_query: Query<&mut DebugCamera>,
    mut debug: ResMut<DebugTerrain>,
) {
    let mut camera = camera_query.single_mut();
    if input.just_pressed(KeyCode::T) {
        camera.active = !camera.active;
        println!(
            "Toggled the camera movement {}.",
            if camera.active { "on" } else { "off" }
        )
    }

    if input.just_pressed(KeyCode::V) {
        debug.test1 = !debug.test1;
        println!(
            "The {} is now used as the terrain height.",
            if debug.test1 { "DSM" } else { "DTM" }
        )
    }
}
