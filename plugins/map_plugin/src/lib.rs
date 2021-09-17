use crate::{
    bundles::{MapBundle, WaterBundle},
    chunks::systems::{
        poll_chunk_tasks, unload_chunks, update_materials_on_change, update_mesh_on_change,
        update_visible_chunks, Viewer, UPDATE_RATE,
    },
    data::register_inspectable_types,
    pipeline::{add_map_pipeline, MapMaterial},
    water::{
        pipeline::WaterTextures,
        systems::{set_water_textures_sampler, update_water_materials, update_waves},
    },
    water::{
        pipeline::{
            add_water_pipeline, MainCamera, ReflectionCamera, RefractionCamera, WaterMaterial,
            WaterPass, REFLECTION_PASS_CAMERA, REFRACTION_PASS_CAMERA,
        },
        systems::{update_reflection_camera, update_water_level},
    },
};
use bevy::{
    core::{FixedTimestep, Name},
    math::Vec3,
    pbr::{PbrBundle, PointLight},
    prelude::{
        shape::{self, Cube, Plane},
        AddAsset, App, Assets, BuildChildren, Color, Commands, Mesh, PerspectiveCameraBundle,
        Plugin, Res, ResMut, StandardMaterial, SystemSet, Transform,
    },
    render::camera::{ActiveCameras, Camera, PerspectiveProjection},
};

mod bundles;
mod chunks;
pub mod data;
pub mod generation;
mod macros;
mod pipeline;
mod water;

/// A plugin that procedurally generates a map.
pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<MapMaterial>()
            .add_asset::<WaterMaterial>()
            .add_startup_system(setup)
            .add_startup_system(setup_cameras)
            .add_system(update_visible_chunks)
            .add_system(poll_chunk_tasks)
            .add_system_set(
                SystemSet::new()
                    .with_system(update_reflection_camera)
                    .with_system(update_water_level)
                    .with_system(update_water_materials)
                    .with_system(update_waves)
                    .with_system(set_water_textures_sampler),
            )
            .add_system_set(
                SystemSet::new()
                    .with_system(update_materials_on_change)
                    .with_system(update_mesh_on_change)
                    .with_system(unload_chunks)
                    .with_run_criteria(FixedTimestep::step(UPDATE_RATE)),
            );

        // add the render pipeline of the map and the water to the graph
        add_map_pipeline(&mut app.world);
        add_water_pipeline(&mut app.world);

        register_inspectable_types(&mut app.world);
    }
}

/// Creates the map with the water.
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    mut water_materials: ResMut<Assets<WaterMaterial>>,
    water_textures: Res<WaterTextures>,
) {
    // point light
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Icosphere::default())),
            material: standard_materials.add(Color::WHITE.into()),
            transform: Transform::from_xyz(100.0, 40.0, -40.0),
            ..Default::default()
        })
        .insert(PointLight {
            intensity: 50.0,
            ..Default::default()
        });

    // dummy cube
    commands
        .spawn_bundle(PbrBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 80.0, 0.0)),
            mesh: meshes.add(Cube::new(10.0).into()),
            material: standard_materials.add(Color::RED.into()),
            ..Default::default()
        })
        .insert(WaterPass)
        .insert(Name::new("Cube"));

    // water
    let water = commands
        .spawn_bundle(WaterBundle {
            mesh: meshes.add(
                Plane {
                    size: u16::MAX as f32,
                }
                .into(),
            ),
            material: water_materials.add(WaterMaterial {
                dudv_texture: water_textures.dudv_texture.clone(),
                normal_texture: water_textures.normal_texture.clone(),
                ..Default::default()
            }),
            ..Default::default()
        })
        .insert(Name::from("Water"))
        .id();

    // map
    commands
        .spawn_bundle(MapBundle::default())
        .insert(Name::new("Map"))
        .push_children(&[water]);
}

/// Spawns the main, refraction and reflection camera.
fn setup_cameras(mut commands: Commands, mut active_cameras: ResMut<ActiveCameras>) {
    let perspective_projection = PerspectiveProjection {
        far: 4000.0, // matches the near and far value in the water shader
        ..Default::default()
    };

    commands
        .spawn_bundle(PerspectiveCameraBundle {
            camera: Camera {
                name: Some(REFLECTION_PASS_CAMERA.to_string()),
                ..Default::default()
            },
            perspective_projection: perspective_projection.clone(),
            ..Default::default()
        })
        .insert(ReflectionCamera)
        .id();
    active_cameras.add(REFLECTION_PASS_CAMERA);

    let refraction_camera = commands
        .spawn_bundle(PerspectiveCameraBundle {
            camera: Camera {
                name: Some(REFRACTION_PASS_CAMERA.to_string()),
                ..Default::default()
            },
            perspective_projection: perspective_projection.clone(),
            ..Default::default()
        })
        .insert(RefractionCamera)
        .id();
    active_cameras.add(REFRACTION_PASS_CAMERA);

    // spawn the main pass camera and add the other two as children
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_translation(Vec3::new(250.0, 25.0, 0.0))
                .looking_at(Vec3::new(-350.0, -75.0, 0.0), Vec3::Y),
            perspective_projection,
            ..Default::default()
        })
        .insert(Viewer) // mark as a viewer
        .insert(MainCamera) // mark as the main camera
        .push_children(&[refraction_camera]);
}
