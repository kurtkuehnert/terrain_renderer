use crate::{
    bundles::{MapBundle, WaterBundle},
    chunks::systems::{
        poll_chunk_tasks, unload_chunks, update_materials_on_change, update_mesh_on_change,
        update_visible_chunks, Viewer, UPDATE_RATE,
    },
    data::register_inspectable_types,
    pipeline::{add_map_pipeline, MapMaterial},
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
    pbr::PbrBundle,
    prelude::{
        shape::{Cube, Plane},
        AddAsset, App, Assets, BuildChildren, Color, Commands, Mesh, PerspectiveCameraBundle,
        Plugin, ResMut, StandardMaterial, SystemSet, Transform,
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
            .add_system(update_reflection_camera)
            .add_system(update_water_level)
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
    mut materials: ResMut<Assets<WaterMaterial>>,
    mut m: ResMut<Assets<StandardMaterial>>,
) {
    let water = commands
        .spawn_bundle(WaterBundle {
            mesh: meshes.add(
                Plane {
                    size: u16::MAX as f32,
                }
                .into(),
            ),
            material: materials.add(WaterMaterial::default()),
            ..Default::default()
        })
        .insert(Name::from("Water"))
        .id();

    commands
        .spawn_bundle(MapBundle::default())
        .insert(Name::new("Map"))
        .push_children(&[water]);

    commands
        .spawn_bundle(PbrBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 80.0, 0.0)),
            mesh: meshes.add(Cube::new(10.0).into()),
            material: m.add(Color::RED.into()),
            ..Default::default()
        })
        .insert(WaterPass)
        .insert(Name::new("Cube"));
}

/// Spawns the main, refraction and reflection camera.
fn setup_cameras(mut commands: Commands, mut active_cameras: ResMut<ActiveCameras>) {
    let perspective_projection = PerspectiveProjection {
        far: 4000.0,
        ..Default::default()
    };

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

    let reflection_camera = commands
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
        .push_children(&[refraction_camera, reflection_camera]);
}
