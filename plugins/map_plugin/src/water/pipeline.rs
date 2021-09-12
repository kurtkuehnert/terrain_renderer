use bevy::{
    prelude::{AssetServer, Assets, Color, Handle, HandleUntyped, Shader, Texture, World},
    reflect::TypeUuid,
    render::{
        pass::{
            LoadOp, Operations, PassDescriptor, RenderPassColorAttachment,
            RenderPassDepthStencilAttachment, TextureAttachment,
        },
        pipeline::PipelineDescriptor,
        render_graph::{
            base::{self, node::MAIN_PASS},
            AssetRenderResourcesNode, CameraNode, PassNode, RenderGraph, TextureNode,
        },
        renderer::RenderResources,
        shader::ShaderStages,
        texture::{
            Extent3d, SamplerDescriptor, TextureDescriptor, TextureDimension, TextureFormat,
            TextureUsage,
        },
    },
};

/// Marks the entity to be rendered in the water pass.
/// Thus it will be relected on the water surface.
#[derive(Default)]
pub struct WaterPass;

/// Marks the main camera, which should have a refractrive and reflective camera as children.
pub struct MainCamera;

/// Marks the refraction camera.
pub struct RefractionCamera;

/// Marks the reflection camera.
pub struct ReflectionCamera;

/// The resolution of the refraction and reflection texture.
const WATER_RESOLUTION: f32 = 2000.0;

pub const REFRACTION_PASS: &str = "refraction_pass";
pub const REFRACTION_PASS_CAMERA: &str = "refraction_pass_camera";
const REFRACTION_TEXTURE_NODE: &str = "refraction_texure_node";
const REFRACTION_DEPTH_TEXTURE_NODE: &str = "refraction_depth_texure_node";

pub const REFLECTION_PASS: &str = "reflection_pass";
pub const REFLECTION_PASS_CAMERA: &str = "reflection_pass_camera";
const REFLECTION_TEXTURE_NODE: &str = "reflection_texure_node";
const REFLECTION_DEPTH_TEXTURE_NODE: &str = "reflection_depth_texure_node";

const COLOR_ATTACHMENT: &str = "color_attachment";
const DEPTH: &str = "depth";

/// The file path of the vertex shader
const VERTEX_SHADER: &str = "shaders/water/vertex.vert";
/// The file path of the fragment shader
const FRAGMENT_SHADER: &str = "shaders/water/fragment.frag";

/// The global handle used for accessing the water pipeline.
pub const WATER_PIPELINE_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(PipelineDescriptor::TYPE_UUID, 18237612412626179);
pub const REFRACTION_TEXTURE_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Texture::TYPE_UUID, 6378234523452345912);
pub const REFLECTION_TEXTURE_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Texture::TYPE_UUID, 13378939762009864029);

/// Sets up the graph and adds the pipeline used to render the water.
pub fn add_water_pipeline(world: &mut World) {
    // load shaders
    let asset_server = world.get_resource_mut::<AssetServer>().unwrap();
    asset_server.watch_for_changes().unwrap();
    let vertex_shader: Handle<Shader> = asset_server.load(VERTEX_SHADER);
    let fragment_shader: Handle<Shader> = asset_server.load(FRAGMENT_SHADER);

    let mut pipelines = world
        .get_resource_mut::<Assets<PipelineDescriptor>>()
        .unwrap();

    // create a new shader pipeline with vertex and fragment shader for the water
    let pipeline = PipelineDescriptor::default_config(ShaderStages {
        vertex: vertex_shader,
        fragment: Some(fragment_shader),
    });

    // assign the pipeline to the constant handle
    pipelines.set_untracked(WATER_PIPELINE_HANDLE, pipeline);

    // add the water material to the render graph
    let graph = &mut *world.get_resource_mut::<RenderGraph>().unwrap();
    WaterMaterial::add_to_graph(graph);

    let size = Extent3d::new(WATER_RESOLUTION as u32, WATER_RESOLUTION as u32, 1);

    // add the refraction and reflaction pass the the render graph
    add_pass(
        graph,
        size,
        REFRACTION_PASS,
        REFRACTION_PASS_CAMERA,
        REFRACTION_TEXTURE_NODE,
        REFRACTION_DEPTH_TEXTURE_NODE,
        REFRACTION_TEXTURE_HANDLE,
    );
    add_pass(
        graph,
        size,
        REFLECTION_PASS,
        REFLECTION_PASS_CAMERA,
        REFLECTION_TEXTURE_NODE,
        REFLECTION_DEPTH_TEXTURE_NODE,
        REFLECTION_TEXTURE_HANDLE,
    );
}

/// Creates and adds the refraction or reflection pass to the render graph.
/// The textures and the corresponding camera are therefore connected to the graph.
fn add_pass(
    graph: &mut RenderGraph,
    size: Extent3d,
    pass: &'static str,
    pass_camera: &'static str,
    texture_node: &'static str,
    depth_texture_node: &'static str,
    texture_handle: HandleUntyped,
) {
    // create a new render pass
    let mut pass_node = PassNode::<&WaterPass>::new(PassDescriptor {
        color_attachments: vec![RenderPassColorAttachment {
            attachment: TextureAttachment::Input(COLOR_ATTACHMENT.to_string()),
            resolve_target: None,
            ops: Operations {
                load: LoadOp::Clear(Color::rgb(0.0, 0.5, 1.0)), // Color::rgba(0.0, 0.0, 0.0, 0.0)),
                store: true,
            },
        }],
        depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
            attachment: TextureAttachment::Input(DEPTH.to_string()),
            depth_ops: Some(Operations {
                load: LoadOp::Clear(1.0),
                store: true,
            }),
            stencil_ops: None,
        }),
        sample_count: 1,
    });

    // set the camera for the pass
    pass_node.add_camera(pass_camera);

    // add the pass to the graph
    graph.add_node(pass, pass_node);

    // add a camera node to the graph and connect it to the pass
    graph.add_system_node(pass_camera, CameraNode::new(pass_camera));
    graph.add_node_edge(pass_camera, pass).unwrap();

    // add a texture and a depth texture to the graph
    graph.add_node(
        texture_node,
        TextureNode::new(
            TextureDescriptor {
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: Default::default(),
                usage: TextureUsage::OUTPUT_ATTACHMENT | TextureUsage::SAMPLED,
            },
            Some(SamplerDescriptor::default()),
            Some(texture_handle),
        ),
    );
    graph.add_node(
        depth_texture_node,
        TextureNode::new(
            TextureDescriptor {
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Depth32Float,
                usage: TextureUsage::OUTPUT_ATTACHMENT | TextureUsage::SAMPLED,
            },
            None,
            None,
        ),
    );

    // connect the texture and depth texture to the pass
    graph
        .add_slot_edge(texture_node, TextureNode::TEXTURE, pass, COLOR_ATTACHMENT)
        .unwrap();
    graph
        .add_slot_edge(depth_texture_node, TextureNode::TEXTURE, pass, DEPTH)
        .unwrap();

    // connect the texture to the pass and connect the pass to the main pass
    graph.add_node_edge(texture_node, pass).unwrap();
    graph.add_node_edge(pass, MAIN_PASS).unwrap();
}

/// The material of the water.
#[derive(RenderResources, TypeUuid)]
#[uuid = "37955fd8-92b7-4203-9517-ab8b4cb35863"]
// #[render_resources(from_self)]
pub struct WaterMaterial {
    pub refraction_texture: Handle<Texture>,
    pub reflection_texture: Handle<Texture>,
}

impl WaterMaterial {
    /// The name of the water material node in the render graph.
    const NODE: &'static str = "water_material_node";

    // Adds the water material to the graph and ensures it is correctly bound to the shader.
    fn add_to_graph(graph: &mut RenderGraph) {
        // add a new water node to the graph
        // this binds the material to the shader
        graph.add_system_node(
            Self::NODE,
            AssetRenderResourcesNode::<WaterMaterial>::new(true),
        );

        // add a new edge to the graph
        // this ensures the water material node runs before the main pass
        graph
            .add_node_edge(Self::NODE, base::node::MAIN_PASS)
            .unwrap();
    }
}

impl Default for WaterMaterial {
    fn default() -> Self {
        Self {
            refraction_texture: REFRACTION_TEXTURE_HANDLE.typed(),
            reflection_texture: REFLECTION_TEXTURE_HANDLE.typed(),
        }
    }
}
