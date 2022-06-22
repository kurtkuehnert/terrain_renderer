use bevy::prelude::*;
use bevy::render::camera::{Projection, Viewport};
use bevy::window::{WindowId, WindowResized};
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
    let perspective_projection = PerspectiveProjection {
        far: 10000.0,
        ..default()
    };

    commands
        .spawn_bundle(Camera3dBundle {
            camera: Camera::default(),
            projection: Projection::Perspective(perspective_projection.clone()),
            transform: Transform::from_xyz(300.0, 750.0, 300.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        })
        .insert(LeftCamera)
        .insert(FlyCamera {
            accel: 8.0,
            friction: 3.0,
            max_speed: 16.0,
            sensitivity: 30.0,
            key_forward: KeyCode::Up,
            key_backward: KeyCode::Down,
            key_left: KeyCode::Left,
            key_right: KeyCode::Right,
            key_up: KeyCode::PageUp,
            key_down: KeyCode::PageDown,
            enabled: false,
            ..default()
        });

    // commands
    //     .spawn_bundle(Camera3dBundle {
    //         camera: Camera::default(),
    //         projection: Projection::Perspective(perspective_projection.clone()),
    //         transform: Transform::from_xyz(1300.0, 750.0, 300.0).looking_at(Vec3::ZERO, Vec3::Y),
    //         ..default()
    //     })
    //     .insert(RightCamera);
}

#[derive(Component)]
pub struct LeftCamera;

#[derive(Component)]
pub struct RightCamera;

pub fn set_camera_viewports(
    windows: Res<Windows>,
    mut resize_events: EventReader<WindowResized>,
    mut left_camera: Query<&mut Camera, (With<LeftCamera>, Without<RightCamera>)>,
    mut right_camera: Query<&mut Camera, With<RightCamera>>,
) {
    // We need to dynamically resize the camera's viewports whenever the window size changes
    // so then each camera always takes up half the screen.
    // A resize_event is sent when the window is first created, allowing us to reuse this system for initial setup.
    for resize_event in resize_events.iter() {
        if resize_event.id == WindowId::primary() {
            let window = windows.primary();
            let mut left_camera = left_camera.single_mut();
            left_camera.viewport = Some(Viewport {
                physical_position: UVec2::new(0, 0),
                physical_size: UVec2::new(window.physical_width() / 2, window.physical_height()),
                ..default()
            });

            // let mut right_camera = right_camera.single_mut();
            // right_camera.viewport = Some(Viewport {
            //     physical_position: UVec2::new(window.physical_width() / 2, 0),
            //     physical_size: UVec2::new(window.physical_width() / 2, window.physical_height()),
            //     ..default()
            // });
        }
    }
}
