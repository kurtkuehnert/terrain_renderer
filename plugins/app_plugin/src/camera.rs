use bevy::{prelude::*, render::camera::Viewport, window::WindowResized};
use bevy_terrain::prelude::*;

#[derive(Default, Resource)]
pub struct SplitScreenCameras(pub(crate) Vec<Entity>);

pub(crate) fn toggle_camera(
    mut current_camera: Local<usize>,
    input: Res<Input<KeyCode>>,
    cameras: Res<SplitScreenCameras>,
    mut camera_query: Query<&mut DebugCamera>,
) {
    if input.just_pressed(KeyCode::T) {
        if *current_camera != 0 {
            camera_query
                .get_mut(cameras.0[*current_camera - 1])
                .unwrap()
                .active = false;
        }

        *current_camera = (*current_camera + 1) % (cameras.0.len() + 1);

        if *current_camera != 0 {
            camera_query
                .get_mut(cameras.0[*current_camera - 1])
                .unwrap()
                .active = true;
        }
    }
}

pub fn update_camera_viewports(
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
