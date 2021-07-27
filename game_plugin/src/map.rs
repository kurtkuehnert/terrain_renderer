use crate::ENABLE_WIREFRAME;
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::mesh::Indices;
use bevy::render::pipeline::{PipelineDescriptor, PrimitiveTopology};
use bevy::render::render_graph::{base, AssetRenderResourcesNode, RenderGraph};
use bevy::render::renderer::RenderResources;
use bevy::render::shader::{ShaderStage, ShaderStages};
use noise::{NoiseFn, OpenSimplex, Seedable};

pub fn generate_noise_map(width: usize, height: usize, scale: f64, seed: u32) -> Vec<Vec<f32>> {
    let noise = OpenSimplex::new();
    noise.set_seed(seed);

    (0..height)
        .into_iter()
        .map(|y| {
            (0..width)
                .into_iter()
                .map(|x| noise.get([y as f64 / scale, x as f64 / scale]) as f32)
                .collect()
        })
        .collect()
}

pub fn generate_map_mesh(map: Vec<Vec<f32>>) -> Mesh {
    let width = map[0].len() as u32;
    let height = map.len() as u32;
    let size = (width * height) as usize;

    let mut indices = Vec::with_capacity(((width - 1) * (height - 1) * 6) as usize);
    let mut positions = Vec::with_capacity(size);
    // let mut normals = Vec::with_capacity(size);
    // let mut uvs = Vec::with_capacity(size);
    let mut colors = Vec::with_capacity(size);

    let mut vertex_index: u32 = 0;

    for y in 0..height {
        for x in 0..width {
            if x < width - 1 && y < height - 1 {
                if ENABLE_WIREFRAME {
                    indices.push(vertex_index);
                    indices.push(vertex_index + width);
                    indices.push(vertex_index + width);
                    indices.push(vertex_index + width + 1);
                    indices.push(vertex_index + width + 1);
                    indices.push(vertex_index + 1);
                    indices.push(vertex_index + 1);
                    indices.push(vertex_index);
                    indices.push(vertex_index);
                    indices.push(vertex_index + width + 1);
                } else {
                    indices.push(vertex_index);
                    indices.push(vertex_index + width);
                    indices.push(vertex_index + width + 1);
                    indices.push(vertex_index + width + 1);
                    indices.push(vertex_index + 1);
                    indices.push(vertex_index);
                }
            }

            positions.push([x as f32, map[y as usize][x as usize] * 1.0, y as f32]);
            // normals.push([1., 0., 0.]);
            // uvs.push([x as f32 / width as f32, y as f32 / height as f32]);
            colors.push([0., map[y as usize][x as usize], 0.]);

            vertex_index += 1;
        }
    }

    let indices = Indices::U32(indices);

    let mut mesh = if ENABLE_WIREFRAME {
        Mesh::new(PrimitiveTopology::LineList)
    } else {
        Mesh::new(PrimitiveTopology::TriangleList)
    };

    mesh.set_indices(Some(indices));
    mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    // mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    // mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.set_attribute(MapMaterial::ATTRIBUTE_COLOR, colors);

    mesh
}

#[derive(RenderResources, Default, TypeUuid)]
#[uuid = "0320b9b8-b3a3-4baa-8bfa-c94008177b17"]
pub struct MapMaterial {}

impl MapMaterial {
    pub const ATTRIBUTE_COLOR: &'static str = "Vertex_Color";
}

const VERTEX_SHADER: &str = r#"
#version 450
layout(location = 0) in vec3 Vertex_Position;
layout(location = 1) in vec3 Vertex_Color;
layout(location = 0) out vec3 v_color;
layout(set = 0, binding = 0) uniform CameraViewProj {
    mat4 ViewProj;
};
layout(set = 1, binding = 0) uniform Transform {
    mat4 Model;
};
void main() {
    gl_Position = ViewProj * Model * vec4(Vertex_Position, 1.0);
    v_color = Vertex_Color; 
}
"#;

const FRAGMENT_SHADER: &str = r#"
#version 450
layout(location = 0) out vec4 o_Target;
layout(location = 0) in vec3 v_color;
void main() {
    o_Target = vec4(v_color, 1.0);
}
"#;

const MAP_MATERIAL_NODE: &str = "map_material_node";

pub fn add_map_pipeline(
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut shaders: ResMut<Assets<Shader>>,
    mut render_graph: ResMut<RenderGraph>,
) -> Handle<PipelineDescriptor> {
    // Create a new shader pipeline
    let pipeline_handle = pipelines.add(PipelineDescriptor::default_config(ShaderStages {
        vertex: shaders.add(Shader::from_glsl(ShaderStage::Vertex, VERTEX_SHADER)),
        fragment: Some(shaders.add(Shader::from_glsl(ShaderStage::Fragment, FRAGMENT_SHADER))),
    }));

    // Add an AssetRenderResourcesNode to our Render Graph. This will bind MapMaterial resources to our shader
    render_graph.add_system_node(
        MAP_MATERIAL_NODE,
        AssetRenderResourcesNode::<MapMaterial>::new(true),
    );

    // Add a Render Graph edge connecting our new node to the main pass node. This ensures map material node runs before the main pass
    render_graph
        .add_node_edge(MAP_MATERIAL_NODE, base::node::MAIN_PASS)
        .unwrap();

    pipeline_handle
}
