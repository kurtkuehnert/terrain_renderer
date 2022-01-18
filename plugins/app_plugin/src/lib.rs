mod camera;

use crate::camera::{setup_camera, toggle_camera_system};
use bevy::{
    pbr::{Clusters, VisiblePointLights},
    prelude::*,
    render::{primitives::Frustum, view::VisibleEntities},
    utils::HashSet,
};
use bevy_fly_camera::{FlyCamera, FlyCameraPlugin};
use bevy_inspector_egui::{WorldInspectorParams, WorldInspectorPlugin};
use bevy_terrain::{
    bundles::{InstanceBundle, TerrainBundle},
    descriptors::register_inspectable_types,
    material::TerrainMaterialPlugin,
    quadtree::{
        traverse_quadtree, update_quadtree_on_change, update_view_distance_on_change, Quadtree,
    },
};
use std::any::TypeId;

/// A plugin, which sets up the testing application.
pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        // enable asset hot reloading
        app.world
            .get_resource::<AssetServer>()
            .unwrap()
            .watch_for_changes()
            .unwrap();

        let mut ignore_components: HashSet<TypeId> = HashSet::default();
        ignore_components.insert(TypeId::of::<Camera>());
        ignore_components.insert(TypeId::of::<GlobalTransform>());
        ignore_components.insert(TypeId::of::<VisibleEntities>());
        ignore_components.insert(TypeId::of::<Frustum>());
        ignore_components.insert(TypeId::of::<Clusters>());
        ignore_components.insert(TypeId::of::<VisiblePointLights>());
        ignore_components.insert(TypeId::of::<ComputedVisibility>());
        ignore_components.insert(TypeId::of::<Quadtree>());
        ignore_components.insert(TypeId::of::<Children>());
        ignore_components.insert(TypeId::of::<Parent>());
        ignore_components.insert(TypeId::of::<PreviousParent>());
        ignore_components.insert(TypeId::of::<FlyCamera>());

        app.insert_resource(Msaa { samples: 4 })
            .insert_resource(WorldInspectorParams {
                despawnable_entities: true,
                ignore_components,
                ..Default::default()
            })
            .add_plugin(FlyCameraPlugin)
            .add_plugin(WorldInspectorPlugin::new())
            .add_plugin(TerrainMaterialPlugin)
            .add_startup_system(setup_scene)
            .add_startup_system(setup_camera)
            .add_system(update_quadtree_on_change.label("update"))
            .add_system(update_view_distance_on_change.label("update"))
            .add_system(traverse_quadtree.after("update"))
            .add_system(toggle_camera_system);

        register_inspectable_types(app);
    }
}

fn setup_scene(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    let dense = commands
        .spawn_bundle(InstanceBundle::new(&mut meshes, false))
        .id();
    let sparse = commands
        .spawn_bundle(InstanceBundle::new(&mut meshes, true))
        .id();

    commands
        .spawn_bundle(TerrainBundle {
            transform: Transform::from_xyz(-100.0, 0.0, -100.0),
            ..Default::default()
        })
        .push_children(&[dense, sparse]);
}
