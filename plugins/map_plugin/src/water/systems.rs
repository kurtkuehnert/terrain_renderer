use crate::{
    data::MapData,
    unwrap_or,
    water::{
        pipeline::{MainCamera, ReflectionCamera},
        Water,
    },
};
use bevy::{
    math::{EulerRot, Quat, Vec3},
    prelude::{Changed, Children, Query, Transform, With, Without},
};

/// Updates the reflection camera's position by mirrowing the main camera at the water height.
#[allow(clippy::type_complexity)]
pub fn update_reflection_camera(
    map_query: Query<(&Transform, &MapData), (Without<MainCamera>, Without<ReflectionCamera>)>,
    main_camera_query: Query<&Transform, (With<MainCamera>, Changed<Transform>)>,
    mut reflection_camera_query: Query<
        &mut Transform,
        (Without<MainCamera>, With<ReflectionCamera>),
    >,
) {
    let (map_transform, map_data) = unwrap_or!(return, map_query.single());
    let main_transform = unwrap_or!(return, main_camera_query.single());
    let mut reflection_transform = unwrap_or!(return, reflection_camera_query.single_mut());

    // get the pitch of the refraction camera
    let (_yaw, pitch, _roll) = main_transform.rotation.to_euler(EulerRot::YXZ);

    // calculate the offset between the refraction and reflection camera
    // equals twice the distance to the water surface
    let offset = 2.0
        * (main_transform.translation.y
            - map_transform.translation.y
            - map_data.get_water_height());

    // calculate the translation of the reflection camera by rotating the offset by the pitch
    let reflection_translation = Vec3::new(0.0, -offset * pitch.cos(), offset * pitch.sin());

    // calculate the rotation of the reflection camera by rotating against twice the pitch
    let reflection_rotation = Quat::from_euler(EulerRot::YXZ, 0.0, -2.0 * pitch, 0.0);

    *reflection_transform = Transform {
        translation: reflection_translation,
        rotation: reflection_rotation,
        ..*reflection_transform
    };
}

/// Updates the water material's water height if the map data changes.
pub fn update_water_level(
    map_query: Query<(&Children, &MapData), Changed<MapData>>,
    mut water_query: Query<&mut Transform, With<Water>>,
) {
    let (water, map_data) = unwrap_or!(return, map_query.single());

    for &water_entity in water.iter() {
        let mut transform = unwrap_or!(continue, water_query.get_mut(water_entity));

        transform.translation.y = map_data.get_water_height();
    }
}
