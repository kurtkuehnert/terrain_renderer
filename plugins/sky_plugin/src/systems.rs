use crate::{pipeline::SkyMaterial, Moon, Sky, SkyMaterialData, Sun};
use bevy::{
    core::Time,
    math::{Quat, Vec3},
    prelude::{Assets, Changed, Handle, Query, Res, ResMut, Transform, With, Without},
};
use std::f32::consts::TAU;
use util::unwrap_or;

#[allow(clippy::type_complexity)]
pub fn rotate_sky(
    time: Res<Time>,
    mut materials: ResMut<Assets<SkyMaterial>>,
    mut sky_query: Query<
        (&mut Transform, &mut Sky, &Handle<SkyMaterial>),
        (Without<Sun>, Without<Moon>),
    >,
    mut sun_query: Query<(&mut Transform, &mut Sun), (Without<Sky>, Without<Moon>)>,
    mut moon_query: Query<&mut Transform, (With<Moon>, Without<Sun>, Without<Sky>)>,
) {
    let delta = time.delta_seconds();

    let (mut sky_transform, mut sky, material) = unwrap_or!(return, sky_query.get_single_mut());
    // rotate the stars on the sky
    sky.cycle = (sky.cycle + delta / sky.period * TAU) % (TAU);
    sky_transform.rotation = Quat::from_axis_angle(sky.rotation_axis, sky.cycle);

    let (mut sun_transform, mut sun) = unwrap_or!(return, sun_query.get_single_mut());
    let mut moon_transform = unwrap_or!(return, moon_query.get_single_mut());

    // rotate the sun and the moon around on the sky
    sun.cycle = (sun.cycle + delta / sun.period * TAU) % (TAU);
    let sun_direction = Quat::from_axis_angle(sun.rotation_axis, sun.cycle) * Vec3::Y;

    let translation = sun_direction * sky.radius;
    sun_transform.translation = sky_transform.translation + translation;
    moon_transform.translation = sky_transform.translation - translation;

    if let Some(material) = materials.get_mut(material) {
        material.set_sun_direction(sun_direction);
    };
}

pub fn update_sky_materials(
    mut materials: ResMut<Assets<SkyMaterial>>,
    sky_query: Query<(&SkyMaterialData, &Handle<SkyMaterial>), Changed<SkyMaterialData>>,
) {
    for (data, material) in sky_query.iter() {
        if let Some(material) = materials.get_mut(material) {
            material.update(data);
        };
    }
}
