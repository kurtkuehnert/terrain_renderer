use bevy::prelude::{App, ClearColor, Color, Msaa, WindowDescriptor};
use bevy::DefaultPlugins;
use game_plugin::GamePlugin;

fn main() {
    App::build()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Color::rgb(0.058, 0.078, 0.098)))
        .insert_resource(WindowDescriptor {
            width: 800.,
            height: 600.,
            title: "Map Generation".to_string(),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(GamePlugin)
        .run();
}
