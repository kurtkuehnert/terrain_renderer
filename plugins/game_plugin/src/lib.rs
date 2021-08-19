use bevy::prelude::*;
use bevy_inspector_egui::{WorldInspectorParams, WorldInspectorPlugin};
use map_plugin::MapPlugin;

/// A plugin containing the entire game.
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(WorldInspectorPlugin::new())
            .add_plugin(MapPlugin)
            .insert_resource(WorldInspectorParams {
                despawnable_entities: true,
                ..Default::default()
            })
            .add_startup_system(setup);
    }
}

/// Creates the main camera for the game.
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // camera
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(-30.0, 5.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });

    // point light
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Icosphere {
                radius: 1.0,
                subdivisions: 4,
            })),
            material: materials.add(Color::WHITE.into()),
            transform: Transform::from_xyz(50.0, 0.0, 0.0),
            ..Default::default()
        })
        .insert(PointLight {
            intensity: 80.0,
            range: 100.0,
            ..Default::default()
        });

    // point light
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Icosphere {
                radius: 1.0,
                subdivisions: 4,
            })),
            material: materials.add(Color::WHITE.into()),
            transform: Transform::from_xyz(30.0, 0.0, 30.0),
            ..Default::default()
        })
        .insert(PointLight {
            intensity: 20.0,
            range: 100.0,
            color: Color::YELLOW,
            ..Default::default()
        });
}
