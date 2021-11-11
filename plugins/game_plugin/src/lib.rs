use bevy::{
    core::Name,
    math::Vec3,
    pbr::{PbrBundle, PointLight},
    prelude::*,
};
use bevy_inspector_egui::{WorldInspectorParams, WorldInspectorPlugin};
use map_plugin::{
    bundles::{MapBundle, WaterBundle},
    pipeline::MapMaterial,
    water::pipeline::{WaterMaterial, WaterPass, WaterTextures},
    MapPlugin,
};
use sky_plugin::{
    cube_sphere::Cubesphere, pipeline::SkyMaterial, Moon, Sky, SkyBundle, SkyMaterialData,
    SkyPlugin, Sun,
};

/// A plugin containing the entire game.
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        // enable asset hot reloading
        app.world
            .get_resource::<AssetServer>()
            .unwrap()
            .watch_for_changes()
            .unwrap();

        app.insert_resource(WorldInspectorParams {
            sort_components: true,
            despawnable_entities: true,
            ..Default::default()
        })
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(SkyPlugin)
        .add_plugin(MapPlugin)
        .add_startup_system(setup_scene);
    }
}

/// Creates the map with the water.
fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    mut map_materials: ResMut<Assets<MapMaterial>>,
    mut water_materials: ResMut<Assets<WaterMaterial>>,
    mut sky_materials: ResMut<Assets<SkyMaterial>>,
    water_textures: Res<WaterTextures>,
) {
    let sky = false;
    let map = true;
    let water = true;

    // sun
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Icosphere {
                radius: 5.0,
                subdivisions: 3,
            })),
            material: standard_materials.add(Color::YELLOW.into()),
            transform: Transform::from_translation(Vec3::new(0.0, 100.0, 0.0)),
            ..Default::default()
        })
        .insert(PointLight {
            intensity: 50.0,
            ..Default::default()
        })
        .insert(Sun {
            rotation_axis: Vec3::Z,
            period: 100.0,
            cycle: 0.0,
        })
        .insert(Name::new("Sun"));

    if sky {
        // moon
        commands
            .spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Icosphere {
                    radius: 2.0,
                    subdivisions: 3,
                })),
                material: standard_materials.add(Color::WHITE.into()),
                ..Default::default()
            })
            .insert(PointLight {
                intensity: 10.0,
                ..Default::default()
            })
            .insert(Moon)
            .insert(Name::new("Moon"));

        // sky
        commands
            .spawn_bundle(SkyBundle {
                mesh: meshes.add(
                    Cubesphere {
                        radius: 1000.0,
                        grid_size: 10,
                    }
                    .into(),
                ),
                material: sky_materials.add(SkyMaterial::default()),
                sky: Sky {
                    radius: 1000.0,
                    rotation_axis: Vec3::new(1.0, 2.0, 3.0).normalize(),
                    period: 500.0,
                    cycle: 0.0,
                },
                ..Default::default()
            })
            .insert(SkyMaterialData::default())
            .insert(WaterPass)
            .insert(Name::new("Sky")); //.insert(Wireframe);
    }

    let mut water_entity = Entity::new(0);

    if water {
        // dummy cube
        commands
            .spawn_bundle(PbrBundle {
                transform: Transform::from_translation(Vec3::new(0.0, 80.0, 0.0)),
                mesh: meshes.add(shape::Cube::new(10.0).into()),
                material: standard_materials.add(Color::RED.into()),
                ..Default::default()
            })
            .insert(WaterPass)
            .insert(Name::new("Cube"));

        // water
        water_entity = commands
            .spawn_bundle(WaterBundle {
                mesh: meshes.add(
                    shape::Plane {
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
    }

    if map {
        if water {
            commands
                .spawn_bundle(MapBundle {
                    map_material: map_materials.add(MapMaterial::default()),
                    ..Default::default()
                })
                .insert(Name::new("Map"))
                .push_children(&[water_entity]);
        } else {
            commands
                .spawn_bundle(MapBundle {
                    map_material: map_materials.add(MapMaterial::default()),
                    ..Default::default()
                })
                .insert(Name::new("Map"));
        }
    }
}
