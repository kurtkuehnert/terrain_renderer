use bevy::{
    prelude::*,
    render::{
        camera::{CameraPlugin, CameraProjection},
        primitives::Frustum,
    },
};
use bevy_fly_camera::FlyCamera;
use bevy_terrain::quadtree::ViewDistance;

pub(crate) fn toggle_camera_system(
    input: Res<Input<KeyCode>>,
    mut camera_query: Query<&mut FlyCamera>,
) {
    if input.just_pressed(KeyCode::T) {
        for mut camera in camera_query.iter_mut() {
            camera.enabled = !camera.enabled;
        }
    }
}

pub(crate) fn setup_camera(mut commands: Commands) {
    let perspective_projection = PerspectiveProjection {
        far: 10000.0,
        ..default()
    };
    let view_projection = perspective_projection.get_projection_matrix();
    let frustum = Frustum::from_view_projection(
        &view_projection,
        &Vec3::ZERO,
        &Vec3::Z,
        perspective_projection.far(),
    );

    commands
        .spawn_bundle(PerspectiveCameraBundle {
            camera: Camera {
                name: Some(CameraPlugin::CAMERA_3D.to_string()),
                near: perspective_projection.near,
                far: perspective_projection.far,
                ..default()
            },
            perspective_projection,
            frustum,
            transform: Transform::from_xyz(-300.0, 150.0, -300.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        })
        .insert(FlyCamera {
            accel: 2.0,
            max_speed: 1.0,
            sensitivity: 30.0,
            key_forward: KeyCode::Up,
            key_backward: KeyCode::Down,
            key_left: KeyCode::Left,
            key_right: KeyCode::Right,
            key_up: KeyCode::LShift,
            key_down: KeyCode::LControl,
            enabled: false,
            ..default()
        })
        .insert(ViewDistance::default());
}
