use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::pipeline::PipelineDescriptor;
use bevy::render::render_graph::{base, AssetRenderResourcesNode, RenderGraph};
use bevy::render::renderer::RenderResources;
use bevy::render::shader::ShaderStages;

/// The material of a map, with a custom vertex color attribute.
#[derive(RenderResources, Default, TypeUuid)]
#[uuid = "0320b9b8-b3a3-4baa-8bfa-c94008177b17"]
pub struct MapMaterial {}

impl MapMaterial {
    /// The name of the shader attribute, which specifies the color of each map vertex.
    pub const ATTRIBUTE_COLOR: &'static str = "Vertex_Color";
}

/// The pipeline used to render a map.
#[derive(Clone)]
pub struct MapPipeline {
    /// The handle for retrieving the pipeline.
    pub pipeline: Handle<PipelineDescriptor>,
}

impl MapPipeline {
    /// The name of the map material node in the render graph.
    const MAP_MATERIAL_NODE: &'static str = "map_material_node";
    // The file path of the vertex shader
    const VERTEX_SHADER: &'static str = "shaders/map/vertex.vert";
    // The file path of the fragment shader
    const FRAGMENT_SHADER: &'static str = "shaders/map/fragment.frag";
}

impl FromWorld for MapPipeline {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource_mut::<AssetServer>().unwrap();

        // watch for changes
        asset_server.watch_for_changes().unwrap();

        // load shaders
        let vertex_shader: Handle<Shader> = asset_server.load(Self::VERTEX_SHADER);
        let fragment_shader: Handle<Shader> = asset_server.load(Self::FRAGMENT_SHADER);

        let mut pipelines = world
            .get_resource_mut::<Assets<PipelineDescriptor>>()
            .unwrap();

        // create a new shader pipeline with vertex and fragment shader for the map
        let pipeline = pipelines.add(PipelineDescriptor::default_config(ShaderStages {
            vertex: vertex_shader,
            fragment: Some(fragment_shader),
        }));

        let mut render_graph = world.get_resource_mut::<RenderGraph>().unwrap();

        // add an AssetRenderResourcesNode to the render graph
        // this binds MapMaterial resources to the shader
        render_graph.add_system_node(
            Self::MAP_MATERIAL_NODE,
            AssetRenderResourcesNode::<MapMaterial>::new(true),
        );

        // add a render graph edge connecting the new node to the main pass node
        // this ensures map material node runs before the main pass
        render_graph
            .add_node_edge(Self::MAP_MATERIAL_NODE, base::node::MAIN_PASS)
            .unwrap();

        MapPipeline { pipeline }
    }
}
