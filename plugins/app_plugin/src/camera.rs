use bevy::{prelude::*, render::camera::Viewport, window::WindowResized};
use bevy_fly_camera::FlyCamera;
use smooth_bevy_cameras::controllers::fps::FpsCameraController;

pub(crate) fn toggle_camera_system(
    input: Res<Input<KeyCode>>,
    mut camera_query: Query<&mut FpsCameraController>,
) {
    if input.just_pressed(KeyCode::T) {
        for mut camera in camera_query.iter_mut() {
            camera.enabled = !camera.enabled;
        }
    }
}

pub(crate) fn setup_camera(mut _commands: Commands) {

    // commands
    //     .spawn_bundle(Camera3dBundle {
    //         camera: Camera::default(),
    //         projection: Projection::Perspective(perspective_projection.clone()),
    //         transform: Transform::from_xyz(1300.0, 750.0, 300.0).looking_at(Vec3::ZERO, Vec3::Y),
    //         ..default()
    //     })
    //     .insert(RightCamera);
}

#[derive(Default, Component)]
pub struct SplitScreenCameras(pub(crate) Vec<Entity>);

pub fn set_camera_viewports(
    windows: Res<Windows>,
    cameras: Res<SplitScreenCameras>,
    mut resize_events: EventReader<WindowResized>,
    mut camera_query: Query<&mut Camera>,
) {
    for _resize_event in resize_events.iter() {
        let window = windows.primary();

        if cameras.0.len() == 1 {
            let mut camera = camera_query.get_mut(cameras.0[0]).unwrap();
            camera.viewport = Some(Viewport {
                physical_position: UVec2::new(0, 0),
                physical_size: UVec2::new(window.physical_width(), window.physical_height()),
                ..default()
            });
        } else if cameras.0.len() == 2 {
            let mut left_camera = camera_query.get_mut(cameras.0[0]).unwrap();
            left_camera.viewport = Some(Viewport {
                physical_position: UVec2::new(0, 0),
                physical_size: UVec2::new(window.physical_width() / 2, window.physical_height()),
                ..default()
            });

            let mut right_camera = camera_query.get_mut(cameras.0[1]).unwrap();
            right_camera.viewport = Some(Viewport {
                physical_position: UVec2::new(window.physical_width() / 2, 0),
                physical_size: UVec2::new(window.physical_width() / 2, window.physical_height()),
                ..default()
            });
        }
    }
}
