use bevy::prelude::*;
use bevy::DefaultPlugins;
use game_plugin::GamePlugin;

fn main() {
    App::build()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Color::rgb(0.058, 0.078, 0.098)))
        .insert_resource(WindowDescriptor {
            width: 1400.,
            height: 1000.,
            title: "Map Generation".to_string(),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(GamePlugin)
        .add_startup_system(window_setup.system())
        .run();
}

fn window_setup(mut windows: ResMut<Windows>) {
    let window = windows.get_primary_mut().unwrap();
    window.set_position(IVec2::new(2000, 100));
}
