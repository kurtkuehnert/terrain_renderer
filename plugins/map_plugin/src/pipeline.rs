use crate::data::{MapAppearanceData, MapTopologyData};
use bevy::{
    core::Bytes,
    prelude::*,
    reflect::TypeUuid,
    render::{
        pipeline::PipelineDescriptor,
        render_graph::{base, AssetRenderResourcesNode, RenderGraph},
        renderer::{RenderResource, RenderResources},
        shader::ShaderStages,
    },
};

/// The file path of the vertex shader
const VERTEX_SHADER: &str = "shaders/map/vertex.vert";
/// The file path of the fragment shader
const FRAGMENT_SHADER: &str = "shaders/map/fragment.frag";

/// The global handle used for accessing the map pipeline.
pub const MAP_PIPELINE_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(PipelineDescriptor::TYPE_UUID, 9022212867101219114);

/// Sets up the graph and adds the pipeline used to render a map.
pub fn add_map_pipeline(world: &mut World) {
    // load shaders
    let asset_server = world.get_resource_mut::<AssetServer>().unwrap();
    let vertex_shader: Handle<Shader> = asset_server.load(VERTEX_SHADER);
    let fragment_shader: Handle<Shader> = asset_server.load(FRAGMENT_SHADER);

    let mut pipelines = world
        .get_resource_mut::<Assets<PipelineDescriptor>>()
        .unwrap();

    // create a new shader pipeline with vertex and fragment shader for the map
    let pipeline = PipelineDescriptor::default_config(ShaderStages {
        vertex: vertex_shader,
        fragment: Some(fragment_shader),
    });

    // assign the pipeline to the constant handle
    pipelines.set_untracked(MAP_PIPELINE_HANDLE, pipeline);

    // add the map material and the clip uniform to the render graph
    let graph = &mut *world.get_resource_mut::<RenderGraph>().unwrap();
    MapMaterial::add_to_graph(graph);
}

/// The material of a map, with a custom vertex color attribute.
#[derive(Default, Debug, RenderResources, TypeUuid)]
#[uuid = "94990167-fc98-4082-a87e-e4473df026e0"]
pub struct MapMaterial {
    height_map: Handle<Texture>,
    appearance: Appearance,
}

impl MapMaterial {
    /// The name of the map material node in the render graph.
    const NODE: &'static str = "map_material_node";

    pub fn update_topology(
        &mut self,
        height_map: Handle<Texture>,
        topology_data: &MapTopologyData,
    ) {
        self.height_map = height_map;
        self.appearance.update_topology(topology_data);
    }

    pub fn update_appearance(&mut self, appearance_data: &MapAppearanceData) {
        self.appearance.update_appearance(appearance_data);
    }

    // Adds the map material to the graph and ensures it is correctly bound to the shader.
    fn add_to_graph(graph: &mut RenderGraph) {
        // add a new map node to the graph
        // this binds the material to the shader
        graph.add_system_node(
            Self::NODE,
            AssetRenderResourcesNode::<MapMaterial>::new(true),
        );

        // add a new edge to the graph
        // this ensures the map material node runs before the main pass
        graph
            .add_node_edge(Self::NODE, base::node::MAIN_PASS)
            .unwrap();
    }
}

/// The material of a map, with a custom vertex color attribute.
#[derive(Default, Debug, Bytes, RenderResource, RenderResources, TypeUuid)]
#[uuid = "0320b9b8-b3a3-4baa-8bfa-c94008177b17"]
#[render_resources(from_self)]
struct Appearance {
    #[render_resources(buffer)]
    layer_colors: [[f32; 4]; Self::MAX_LAYER_COUNT],
    // uses array of vec4 because the glsl layout for arrays of scalars (floats) has an alignment of vec4 so it is wasting space anyway
    #[render_resources(buffer)]
    layer_heights: [[f32; 4]; Self::MAX_LAYER_COUNT],
    #[render_resources(buffer)]
    blend_values: [[f32; 4]; Self::MAX_LAYER_COUNT],
    map_height: f32,
    water_height: f32,
    layer_count: i32,
}

impl Appearance {
    /// The count of color layers a map can be shaded with.
    /// Corresponds to the value in the fragment shader.
    const MAX_LAYER_COUNT: usize = 16;

    fn update_appearance(&mut self, appearance_data: &MapAppearanceData) {
        appearance_data
            .layer_colors
            .iter()
            .enumerate()
            .for_each(|(i, color)| self.layer_colors[i] = color.as_rgba_f32());

        appearance_data
            .layer_heights
            .iter()
            .enumerate()
            .for_each(|(i, &height)| self.layer_heights[i] = [height, 0.0, 0.0, 0.0]);

        appearance_data
            .blend_values
            .iter()
            .enumerate()
            .for_each(|(i, &blend)| self.blend_values[i] = [blend, 0.0, 0.0, 0.0]);

        self.layer_count = appearance_data.layer_heights.len() as i32;
    }

    fn update_topology(&mut self, topology_data: &MapTopologyData) {
        self.map_height = topology_data.map_height;
        self.water_height = topology_data.water_height()
    }
}
