use crate::data::MaterialData;
use bevy::{
    core::Bytes,
    prelude::{AssetServer, Assets, Handle, HandleUntyped, Shader, World},
    reflect::TypeUuid,
    render::{
        pipeline::PipelineDescriptor,
        render_graph::{base, AssetRenderResourcesNode, RenderGraph},
        renderer::{RenderResource, RenderResources},
        shader::ShaderStages,
    },
};

/// The count of color layers a map can be shaded with.
/// Corresponds to the value in the fragment shader.
const MAX_LAYER_COUNT: usize = 5;

/// The name of the map material node in the render graph.
const MAP_MATERIAL_NODE: &str = "map_material_node";
/// The file path of the vertex shader
const VERTEX_SHADER: &str = "shaders/map/vertex.vert";
/// The file path of the fragment shader
const FRAGMENT_SHADER: &str = "shaders/map/fragment.frag";

/// The global handle used for accessing the map pipeline.
pub const MAP_PIPELINE_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(PipelineDescriptor::TYPE_UUID, 9022212867101219114);

/// Sets up the graph and the pipeline used to render a map.
pub fn add_map_graph(world: &mut World) {
    let asset_server = world.get_resource_mut::<AssetServer>().unwrap();

    // watch for changes
    asset_server.watch_for_changes().unwrap();

    // load shaders
    let vertex_shader: Handle<Shader> = asset_server.load(VERTEX_SHADER);
    let fragment_shader: Handle<Shader> = asset_server.load(FRAGMENT_SHADER);

    let mut render_graph = world.get_resource_mut::<RenderGraph>().unwrap();

    // add an AssetRenderResourcesNode to the render graph
    // this binds MapMaterial resources to the shader
    render_graph.add_system_node(
        MAP_MATERIAL_NODE,
        AssetRenderResourcesNode::<MapMaterial>::new(true),
    );

    // add a render graph edge connecting the new node to the main pass node
    // this ensures map material node runs before the main pass
    render_graph
        .add_node_edge(MAP_MATERIAL_NODE, base::node::MAIN_PASS)
        .unwrap();

    // create a new shader pipeline with vertex and fragment shader for the map
    let pipeline = PipelineDescriptor::default_config(ShaderStages {
        vertex: vertex_shader,
        fragment: Some(fragment_shader),
    });

    let mut pipelines = world
        .get_resource_mut::<Assets<PipelineDescriptor>>()
        .unwrap();

    // assign the pipeline to the constant handle
    pipelines.set_untracked(MAP_PIPELINE_HANDLE, pipeline);
}

/// The material of a map, with a custom vertex color attribute.
#[derive(Bytes, RenderResource, RenderResources, TypeUuid)]
#[uuid = "0320b9b8-b3a3-4baa-8bfa-c94008177b17"]
#[render_resources(from_self)]
pub struct MapMaterial {
    #[render_resources(buffer)]
    pub layer_colors: [[f32; 4]; MAX_LAYER_COUNT],
    // uses array of vec4 because the glsl layout for arrays of scalars (floats) has an alignment of vec4 so it is wasting space anyway
    #[render_resources(buffer)]
    pub layer_heights: [[f32; 4]; MAX_LAYER_COUNT],
    #[render_resources(buffer)]
    pub blend_values: [[f32; 4]; MAX_LAYER_COUNT],
    pub map_height: f32,
    pub layer_count: i32,
}

impl MapMaterial {
    pub fn new(material_data: &MaterialData, map_height: f32) -> Self {
        let mut layer_colors = [[0.0; 4]; MAX_LAYER_COUNT];
        material_data
            .layer_colors
            .iter()
            .enumerate()
            .for_each(|(i, color)| layer_colors[i] = color.as_rgba_f32());

        let mut layer_heights = [[1.0; 4]; MAX_LAYER_COUNT];
        material_data
            .layer_heights
            .iter()
            .enumerate()
            .for_each(|(i, &height)| layer_heights[i] = [height, 0.0, 0.0, 0.0]);

        let mut blend_values = [[0.0; 4]; MAX_LAYER_COUNT];
        material_data
            .blend_values
            .iter()
            .enumerate()
            .for_each(|(i, &blend)| blend_values[i] = [blend, 0.0, 0.0, 0.0]);

        Self {
            layer_colors,
            layer_heights,
            blend_values,
            map_height,
            layer_count: material_data.layer_heights.len() as i32,
        }
    }
}
