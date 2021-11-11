use crate::systems::update_appearance_on_change;
use crate::{
    data::register_inspectable_types,
    pipeline::{add_map_pipeline, MapMaterial},
    systems::{poll_topology_tasks, update_clipmap, update_topology_on_change},
    water::{
        pipeline::{
            add_water_pipeline, MainCamera, ReflectionCamera, RefractionCamera, WaterMaterial,
            REFLECTION_PASS_CAMERA, REFRACTION_PASS_CAMERA,
        },
        systems::{
            set_water_textures_sampler, update_reflection_camera, update_water_level,
            update_water_materials, update_waves,
        },
    },
};
use bevy::{
    core::FixedTimestep,
    math::Vec3,
    prelude::*,
    render::camera::{ActiveCameras, Camera, PerspectiveProjection},
};

pub mod bundles;
pub mod data;
pub mod generation;
pub mod pipeline;
mod systems;
mod tile;
pub mod water;

/// A plugin that procedurally generates a map.
pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<MapMaterial>()
            .add_asset::<WaterMaterial>()
            .add_startup_system(setup_cameras)
            .add_system_set(
                SystemSet::new()
                    .with_system(update_clipmap)
                    .with_system(update_appearance_on_change)
                    .with_system(poll_topology_tasks),
            )
            .add_system_set(
                SystemSet::new()
                    .label("Topology Update")
                    .with_system(update_topology_on_change)
                    .with_run_criteria(FixedTimestep::step(4.0)),
            )
            .add_system_set(
                SystemSet::new()
                    .with_system(update_reflection_camera)
                    .with_system(update_water_level)
                    .with_system(update_water_materials)
                    .with_system(update_waves)
                    .with_system(set_water_textures_sampler),
            );

        // add the render pipeline of the map and the water to the graph
        add_map_pipeline(&mut app.world);
        add_water_pipeline(&mut app.world);

        register_inspectable_types(&mut app.world);
    }
}

// Todo: move out of here
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
            transform: Transform::from_translation(Vec3::new(300.0, 65.0, 65.0))
                .looking_at(Vec3::new(-300.0, -35.0, 65.0), Vec3::Y),
            perspective_projection,
            ..Default::default()
        }) // mark as a viewer
        .insert(MainCamera) // mark as the main camera
        .push_children(&[refraction_camera]);
}
