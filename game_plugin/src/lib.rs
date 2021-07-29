pub mod map;

use crate::map::{add_map_pipeline, Map, MapData, MapMaterial};
use bevy::core::FixedTimestep;
use bevy::prelude::*;
use bevy::render::pipeline::{PipelineDescriptor, RenderPipeline};
use bevy::render::render_graph::RenderGraph;
use bevy_config_cam::ConfigCam;
use bevy_inspector_egui::InspectorPlugin;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(ConfigCam)
            .add_plugin(InspectorPlugin::<MapData>::new())
            .add_asset::<MapMaterial>()
            .add_startup_system(setup.system())
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(FixedTimestep::step(0.5))
                    .with_system(update_map.system()),
            );
    }
}

fn setup(
    mut commands: Commands,
    map_data: Res<MapData>,
    mut meshes: ResMut<Assets<Mesh>>,
    pipelines: ResMut<Assets<PipelineDescriptor>>,
    shaders: ResMut<Assets<Shader>>,
    render_graph: ResMut<RenderGraph>,
) {
    // basic light
    commands.spawn_bundle(LightBundle {
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });

    // prepare the map and the render pipeline of the mesh
    let mesh = meshes.add(map_data.as_mesh());
    let pipeline = add_map_pipeline(pipelines, shaders, render_graph);
    let render_pipelines = RenderPipelines::from_pipelines(vec![RenderPipeline::new(pipeline)]);

    // create the map entity
    commands
        .spawn_bundle(PbrBundle {
            mesh,
            render_pipelines,
            ..Default::default()
        })
        .insert(Map);
}

fn update_map(
    map_data: Res<MapData>,
    mut meshes: ResMut<Assets<Mesh>>,
    query: Query<&Handle<Mesh>, With<Map>>,
) {
    // only update on change
    if !map_data.is_changed() {
        return;
    }

    // regenerate the mesh of the map
    for mesh in query.iter() {
        let mesh = meshes.get_mut(mesh.clone()).unwrap();
        *mesh = map_data.as_mesh();
    }
}
