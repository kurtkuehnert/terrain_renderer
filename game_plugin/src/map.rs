use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::mesh::Indices;
use bevy::render::pipeline::{PipelineDescriptor, PrimitiveTopology};
use bevy::render::render_graph::{base, AssetRenderResourcesNode, RenderGraph};
use bevy::render::renderer::RenderResources;
use bevy::render::shader::{ShaderStage, ShaderStages};
use bevy_inspector_egui::Inspectable;
use nalgebra_glm::smoothstep;
use noise::{NoiseFn, OpenSimplex};
use rand::prelude::*;

pub struct Map;

#[derive(Inspectable)]
pub struct MapData {
    #[inspectable(min = 1, max = 1000)]
    pub width: usize,
    #[inspectable(min = 1, max = 1000)]
    pub height: usize,
    #[inspectable(min = 0.0, max = 100.0)]
    pub map_height: f32,
    #[inspectable(min = 0.0, max = 100.0)]
    pub scale: f64,
    pub seed: u64,
    #[inspectable(min = 1, max = 10)]
    pub octaves: u32,
    #[inspectable(min = 0.0, max = 1.0)]
    pub persistence: f32,
    #[inspectable(min = 1.0, max = 100.0)]
    pub lacunarity: f64,
    pub wireframe: bool,
}

impl Default for MapData {
    fn default() -> Self {
        MapData {
            width: 50,
            height: 50,
            map_height: 5.0,
            scale: 10.0,
            seed: 0,
            octaves: 4,
            persistence: 0.5,
            lacunarity: 3.0,
            wireframe: false,
        }
    }
}

impl MapData {
    pub fn as_mesh(&self) -> Mesh {
        let map_shape = MapShape::new(self);
        map_shape.into()
    }
}

pub struct MapShape<'a> {
    map_data: &'a MapData,
}

impl<'a> MapShape<'a> {
    pub fn new(map_data: &'a MapData) -> Self {
        MapShape { map_data }
    }

    pub fn generate_noise_map(&self) -> Vec<Vec<f32>> {
        let &data = &self.map_data;

        // prepare the noise and the random number generator
        let noise = OpenSimplex::new();
        let mut rng = StdRng::seed_from_u64(data.seed);

        // generate random offsets for each octave
        let octave_offsets: Vec<(f64, f64)> = (0..data.octaves)
            .into_iter()
            .map(|_| {
                (
                    rng.gen_range(-1000.0..1000.0),
                    rng.gen_range(-1000.0..1000.0),
                )
            })
            .collect();

        // init the noise map
        let mut map = Vec::with_capacity(data.height);

        let scale = data.scale.max(0.001); // sanity check the scale
        let mut max_height = f32::MIN;
        let mut min_height = f32::MAX;
        let half_width = data.width as f64 / 2.0;
        let half_height = data.height as f64 / 2.0;

        for y in 0..data.height {
            // init the next row of the map
            let mut row = Vec::with_capacity(data.width);

            for x in 0..data.width {
                let mut amplitude = 1.;
                let mut frequency = 1.;
                let mut noise_height = 0.;

                // sum up the height at the position for all octaves
                for &(offset_x, offset_y) in octave_offsets.iter() {
                    let sample_x = (x as f64 - half_width) / scale * frequency + offset_x;
                    let sample_y = (y as f64 - half_height) / scale * frequency + offset_y;
                    noise_height += noise.get([sample_x, sample_y]) as f32 * amplitude;

                    amplitude *= data.persistence;
                    frequency *= data.lacunarity;
                }

                // adjust the max and min value
                if max_height < noise_height {
                    max_height = noise_height;
                } else if min_height > noise_height {
                    min_height = noise_height;
                }

                row.push(noise_height);
            }

            map.push(row);
        }

        // normalize the map between the min and max value
        for y in 0..data.height {
            for x in 0..data.width {
                map[y][x] = smoothstep(min_height, max_height, map[y][x]) * data.map_height;
            }
        }

        map
    }

    pub fn mesh_attributes(
        &self,
    ) -> (
        Vec<[f32; 3]>,
        Vec<[f32; 3]>,
        Vec<[f32; 2]>,
        Vec<[f32; 3]>,
        Vec<u32>,
    ) {
        let map = self.generate_noise_map();

        let width = map[0].len() as u32;
        let height = map.len() as u32;
        let size = (width * height) as usize;

        let mut positions = Vec::with_capacity(size);
        let mut normals = Vec::with_capacity(size);
        let mut uvs = Vec::with_capacity(size);
        let mut colors = Vec::with_capacity(size);
        let mut indices = Vec::with_capacity(((width - 1) * (height - 1) * 6) as usize);

        let mut vertex_index: u32 = 0;

        for y in 0..height {
            for x in 0..width {
                if x < width - 1 && y < height - 1 {
                    if self.map_data.wireframe {
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
                normals.push([1.0, 0.0, 0.0]);
                uvs.push([x as f32 / width as f32, y as f32 / height as f32]);
                colors.push([
                    0.0,
                    0.0,
                    map[y as usize][x as usize] / self.map_data.map_height,
                ]);

                vertex_index += 1;
            }
        }

        (positions, normals, uvs, colors, indices)
    }
}

impl<'a> From<MapShape<'a>> for Mesh {
    fn from(map_shape: MapShape) -> Self {
        let (positions, normals, uvs, colors, indices) = map_shape.mesh_attributes();

        let mut mesh = if map_shape.map_data.wireframe {
            Mesh::new(PrimitiveTopology::LineList)
        } else {
            Mesh::new(PrimitiveTopology::TriangleList)
        };

        mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.set_attribute(MapMaterial::ATTRIBUTE_COLOR, colors);
        mesh.set_indices(Some(Indices::U32(indices)));
        mesh
    }
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
