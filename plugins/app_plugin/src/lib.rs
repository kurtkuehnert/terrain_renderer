mod camera;

use crate::camera::{setup_camera, toggle_camera_system};
use bevy::{
    pbr::{Clusters, VisiblePointLights},
    prelude::*,
    render::{primitives::Frustum, view::VisibleEntities},
    utils::HashSet,
};
use bevy_fly_camera::FlyCameraPlugin;
use bevy_inspector_egui::{WorldInspectorParams, WorldInspectorPlugin};
use bevy_terrain::{
    bundles::PieceBundle,
    clipmap::{update_hierarchy_on_change, TileHierarchy},
    descriptors::{register_inspectable_types, TileHierarchyDescriptor},
    material::TerrainMaterial,
    pieces::{PieceVariant, StripeVariant, TriangleVariant},
};
use std::{any::TypeId, f32::consts::FRAC_PI_2};

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
        ignore_components.insert(TypeId::of::<TileHierarchy>());

        app.insert_resource(Msaa { samples: 4 })
            .insert_resource(WorldInspectorParams {
                despawnable_entities: true,
                ignore_components,
                ..Default::default()
            })
            .add_plugin(FlyCameraPlugin)
            .add_plugin(WorldInspectorPlugin::new())
            .add_plugin(MaterialPlugin::<TerrainMaterial>::default())
            .add_startup_system(setup_scene)
            .add_startup_system(setup_camera)
            .add_system(update_hierarchy_on_change)
            .add_system(toggle_camera_system);

        register_inspectable_types(app);
    }
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<TerrainMaterial>>,
) {
    let tile_hierarchy = TileHierarchy::new(&mut meshes);

    let triangle_mesh = tile_hierarchy.get_mesh(PieceVariant::Triangle(TriangleVariant::Sparse));
    let stripe_mesh = tile_hierarchy.get_mesh(PieceVariant::Stripe(StripeVariant::Sparse));

    let mut pieces = Vec::new();

    for angle in (0..4).map(|i| i as f32 * FRAC_PI_2) {
        let rotation = Quat::from_axis_angle(Vec3::Y, angle);
        let translation = rotation * Vec3::new(0.0, 0.0, 2.0);
        let scale = Vec3::new(1.0, 1.0, 1.0);

        pieces.push(
            commands
                .spawn()
                .insert_bundle(PieceBundle {
                    mesh: triangle_mesh.clone(),
                    transform: Transform {
                        translation,
                        rotation,
                        scale,
                    },
                    material: materials.add(TerrainMaterial { color: Color::BLUE }),
                    ..Default::default()
                })
                .id(),
        );

        let rotation = Quat::from_axis_angle(Vec3::Y, angle);
        let translation = Vec3::ZERO;
        let scale = Vec3::new(1.0, 1.0, 1.0);

        pieces.push(
            commands
                .spawn()
                .insert_bundle(PieceBundle {
                    mesh: stripe_mesh.clone(),
                    transform: Transform {
                        translation,
                        rotation,
                        scale,
                    },
                    material: materials.add(TerrainMaterial { color: Color::RED }),
                    ..Default::default()
                })
                .id(),
        );
    }

    commands
        .spawn()
        .insert(TileHierarchyDescriptor::default())
        .insert(tile_hierarchy)
        .insert(Transform::default())
        .insert(GlobalTransform::default())
        .push_children(&pieces);
}
