use crate::{
    data::{MapData, WaterMaterialData},
    unwrap_or,
    water::{
        pipeline::{MainCamera, ReflectionCamera, WaterMaterial, WaterTextures},
        Water,
    },
};
use bevy::{
    core::Time,
    math::{EulerRot, Quat, Vec3},
    prelude::{
        AssetEvent, Assets, Changed, EventReader, Handle, Query, Res, ResMut, Texture, Transform,
        With, Without,
    },
    render::texture::{AddressMode, FilterMode, SamplerDescriptor},
};

/// Scales down the wave speed, for a more reasonable range of values.
const WAVE_MODIFIER: f32 = 0.1;

/// Updates the reflection camera's position by mirroring the main camera at the water height.
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
    let (yaw, pitch, roll) = main_transform.rotation.to_euler(EulerRot::YXZ);

    // calculate the offset between the refraction and reflection camera
    // equals twice the distance to the water surface
    let offset = 2.0
        * (main_transform.translation.y
            - map_transform.translation.y
            - map_data.get_water_height());

    // calculate the translation of the reflection camera by offsetting the height under the water
    let reflection_translation = main_transform.translation + Vec3::new(0.0, -offset, 0.0);

    // calculate the rotation of the reflection camera by inverting the pitch and the roll
    let reflection_rotation = Quat::from_euler(EulerRot::YXZ, yaw, -pitch, -roll);

    *reflection_transform = Transform {
        translation: reflection_translation,
        rotation: reflection_rotation,
        ..*reflection_transform
    };
}

/// Updates the water material's water height if the map data changes.
pub fn update_water_level(
    map_query: Query<&MapData, Changed<MapData>>,
    mut water_query: Query<&mut Transform, With<Water>>,
) {
    let data = unwrap_or!(return, map_query.single());

    for mut transform in water_query.iter_mut() {
        transform.translation.y = data.get_water_height();
    }
}

/// Updates the material of the water if the water material data of the map has changed.
pub fn update_water_materials(
    mut materials: ResMut<Assets<WaterMaterial>>,
    map_query: Query<&WaterMaterialData, Changed<WaterMaterialData>>,
    water_query: Query<&Handle<WaterMaterial>>,
) {
    let data = unwrap_or!(return, map_query.single());

    for material in water_query.iter() {
        if let Some(material) = materials.get_mut(material) {
            material.update(data);
        };
    }
}

/// Updates the cycle of the wave.
pub fn update_waves(
    time: Res<Time>,
    mut materials: ResMut<Assets<WaterMaterial>>,
    map_query: Query<&WaterMaterialData>,
    water_query: Query<&Handle<WaterMaterial>>,
) {
    let data = unwrap_or!(return, map_query.single());

    for material in water_query.iter() {
        if let Some(material) = materials.get_mut(material) {
            material.wave_uniform.wave_cycle +=
                time.delta_seconds() * data.wave_speed * WAVE_MODIFIER;
            material.wave_uniform.wave_cycle %= 1.0;
        }
    }
}

/// Sets the sampler for the water textures once they are loaded.
pub fn set_water_textures_sampler(
    mut asset_events: EventReader<AssetEvent<Texture>>,
    mut assets: ResMut<Assets<Texture>>,
    water_textures: Res<WaterTextures>,
) {
    for event in asset_events.iter() {
        if let AssetEvent::Created { handle } = event {
            if *handle == water_textures.dudv_texture || *handle == water_textures.normal_texture {
                let texture = assets.get_mut(handle).unwrap();

                texture.sampler = SamplerDescriptor {
                    address_mode_u: AddressMode::Repeat,
                    address_mode_v: AddressMode::Repeat,
                    address_mode_w: AddressMode::Repeat,
                    mag_filter: FilterMode::Linear,
                    min_filter: FilterMode::Linear,
                    ..texture.sampler
                };
            }
        }
    }
}
