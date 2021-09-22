use bevy::{
    core::Bytes,
    math::Vec3,
    prelude::{AssetServer, Assets, Handle, HandleUntyped, Shader, World},
    reflect::TypeUuid,
    render::{
        pipeline::{FrontFace, PipelineDescriptor},
        render_graph::{base, AssetRenderResourcesNode, RenderGraph},
        renderer::{RenderResource, RenderResources},
        shader::ShaderStages,
    },
};

use crate::SkyMaterialData;

/// The file path of the vertex shader
const VERTEX_SHADER: &str = "shaders/sky/vertex.vert";
/// The file path of the fragment shader
const FRAGMENT_SHADER: &str = "shaders/sky/fragment.frag";

/// The global handle used for accessing the sky pipeline.
pub const SKY_PIPELINE_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(PipelineDescriptor::TYPE_UUID, 523478971259871256);

pub fn add_sky_pipeline(world: &mut World) {
    // load shaders
    let asset_server = world.get_resource_mut::<AssetServer>().unwrap();
    let vertex_shader: Handle<Shader> = asset_server.load(VERTEX_SHADER);
    let fragment_shader: Handle<Shader> = asset_server.load(FRAGMENT_SHADER);

    // create a new shader pipeline with vertex and fragment shader for the sky
    let mut pipeline = PipelineDescriptor::default_config(ShaderStages {
        vertex: vertex_shader,
        fragment: Some(fragment_shader),
    });

    pipeline.primitive.front_face = FrontFace::Cw;

    let mut pipelines = world
        .get_resource_mut::<Assets<PipelineDescriptor>>()
        .unwrap();

    // assign the pipeline to the constant handle
    pipelines.set_untracked(SKY_PIPELINE_HANDLE, pipeline);

    // add the map material and the clip uniform to the render graph
    let graph = &mut *world.get_resource_mut::<RenderGraph>().unwrap();
    SkyMaterial::add_to_graph(graph);
}

/// The material of the sky.
#[derive(Bytes, Default, RenderResource, RenderResources, TypeUuid)]
#[uuid = "d01800e2-63a8-462b-98e9-812574983900"]
#[render_resources(from_self)]
pub struct SkyMaterial {
    sun_direction: [f32; 4],
    sky_color: [f32; 4],
    horizon_color: [f32; 4],
    sun_color: [f32; 4],
    moon_color: [f32; 4],
    star_count: f32,
    star_sharpness: f32,
    sun_size: f32,
    moon_size: f32,
    moon_phase: f32,
}

impl SkyMaterial {
    /// The name of the sky material node in the render graph.
    const NODE: &'static str = "sky_material_node";

    pub fn update(&mut self, data: &SkyMaterialData) {
        *self = Self {
            sky_color: data.sky_color.as_rgba_f32(),
            horizon_color: data.horizon_color.as_rgba_f32(),
            star_count: data.star_count,
            star_sharpness: data.star_sharpness,
            sun_color: data.sun_color.as_rgba_f32(),
            sun_size: data.sun_size,
            moon_color: data.moon_color.as_rgba_f32(),
            moon_size: data.moon_size,
            ..*self
        };
    }

    pub fn set_sun_direction(&mut self, sun_direction: Vec3) {
        self.sun_direction = [sun_direction.x, sun_direction.y, sun_direction.z, 1.0];
    }

    // Adds the sky material to the graph and ensures it is correctly bound to the shader.
    fn add_to_graph(graph: &mut RenderGraph) {
        // add a new sky node to the graph
        // this binds the material to the shader
        graph.add_system_node(
            Self::NODE,
            AssetRenderResourcesNode::<SkyMaterial>::new(true),
        );

        // add a new edge to the graph
        // this ensures the sky material node runs before the main pass
        graph
            .add_node_edge(Self::NODE, base::node::MAIN_PASS)
            .unwrap();
    }
}
