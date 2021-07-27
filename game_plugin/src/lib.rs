pub mod map;

use crate::map::{add_map_pipeline, generate_map_mesh, generate_noise_map, MapMaterial};
use bevy::prelude::*;
use bevy::render::pipeline::{PipelineDescriptor, RenderPipeline};
use bevy::render::render_graph::RenderGraph;
use bevy_config_cam::ConfigCam;

pub struct GamePlugin;

const ENABLE_WIREFRAME: bool = true;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(setup.system())
            .add_plugin(ConfigCam)
            .add_asset::<MapMaterial>();
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    pipelines: ResMut<Assets<PipelineDescriptor>>,
    shaders: ResMut<Assets<Shader>>,
    render_graph: ResMut<RenderGraph>,
) {
    commands.spawn_bundle(LightBundle {
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });

    let map = generate_noise_map(20, 10, 1., 1);
    let mesh = generate_map_mesh(map);

    let handle = meshes.add(mesh);

    let pipeline_handle = add_map_pipeline(pipelines, shaders, render_graph);

    commands.spawn_bundle(PbrBundle {
        mesh: handle,
        render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
            pipeline_handle,
        )]),
        ..Default::default()
    });
}
