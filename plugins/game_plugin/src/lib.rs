use bevy::{
    prelude::{
        shape, App, Assets, Color, Commands, Mesh, PbrBundle, PerspectiveCameraBundle, Plugin,
        PointLight, ResMut, StandardMaterial, Transform, Vec3,
    },
    render::camera::PerspectiveProjection,
};
use bevy_inspector_egui::{WorldInspectorParams, WorldInspectorPlugin};
use map_plugin::MapPlugin;

/// A plugin containing the entire game.
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(WorldInspectorPlugin::new())
            .add_plugin(MapPlugin)
            .insert_resource(WorldInspectorParams {
                sort_components: true,
                despawnable_entities: true,
                ..Default::default()
            })
            .add_startup_system(setup);
    }
}

/// Creates the main camera and some lights for the game.
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // camera
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(1.0, 1500.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
        perspective_projection: PerspectiveProjection {
            far: 4000.0,
            ..Default::default()
        },
        ..Default::default()
    });

    // point light
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Icosphere::default())),
            material: materials.add(Color::WHITE.into()),
            transform: Transform::from_xyz(50.0, 0.0, 0.0),
            ..Default::default()
        })
        .insert(PointLight {
            intensity: 80.0,
            ..Default::default()
        });

    // point light
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Icosphere::default())),
            material: materials.add(Color::WHITE.into()),
            transform: Transform::from_xyz(30.0, 0.0, 30.0),
            ..Default::default()
        })
        .insert(PointLight {
            intensity: 30.0,
            color: Color::ORANGE,
            ..Default::default()
        });
}
