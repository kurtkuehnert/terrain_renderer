use bevy::prelude::{
    shape, App, Assets, Color, Commands, Mesh, PbrBundle, Plugin, PointLight, ResMut,
    StandardMaterial, Transform,
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
    // point light
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Icosphere::default())),
            material: materials.add(Color::WHITE.into()),
            transform: Transform::from_xyz(300.0, 100.0, 0.0),
            ..Default::default()
        })
        .insert(PointLight {
            intensity: 50.0,
            ..Default::default()
        });
}
