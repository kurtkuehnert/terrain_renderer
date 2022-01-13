use bevy::prelude::*;
use bevy_fly_camera::FlyCamera;

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
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_xyz(0.0, 10.0, 25.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        })
        .insert(FlyCamera {
            key_forward: KeyCode::Up,
            key_backward: KeyCode::Down,
            key_left: KeyCode::Left,
            key_right: KeyCode::Right,
            key_up: KeyCode::LShift,
            key_down: KeyCode::LControl,
            enabled: false,
            ..Default::default()
        });
}
