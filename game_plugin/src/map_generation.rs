use crate::map::MapData;
use crate::map_pipeline::MapMaterial;
use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::pipeline::PrimitiveTopology;
use nalgebra_glm::smoothstep;
use noise::{NoiseFn, OpenSimplex};
use rand::prelude::*;

/// Generates a noise map from the supplied map data.
fn generate_noise_map(map_data: &MapData) -> Vec<Vec<f32>> {
    // prepare the noise and the random number generator
    let noise = OpenSimplex::new();
    let mut rng = StdRng::seed_from_u64(map_data.seed);

    // generate random offsets for each octave
    let octave_offsets: Vec<(f64, f64)> = (0..map_data.octaves)
        .into_iter()
        .map(|_| {
            (
                rng.gen_range(-1000.0..1000.0),
                rng.gen_range(-1000.0..1000.0),
            )
        })
        .collect();

    // init the noise map
    let mut map = Vec::with_capacity(map_data.height);

    let scale = map_data.scale.max(0.001); // sanity check the scale
    let mut max_height = f32::MIN;
    let mut min_height = f32::MAX;
    let half_width = map_data.width as f64 / 2.0;
    let half_height = map_data.height as f64 / 2.0;

    for y in 0..map_data.height {
        // init the next row of the map
        let mut row = Vec::with_capacity(map_data.width);

        for x in 0..map_data.width {
            let mut amplitude = 1.;
            let mut frequency = 1.;
            let mut noise_height = 0.;

            // sum up the height at the position for all octaves
            for &(offset_x, offset_y) in octave_offsets.iter() {
                let sample_x = (x as f64 - half_width) / scale * frequency + offset_x;
                let sample_y = (y as f64 - half_height) / scale * frequency + offset_y;
                noise_height += noise.get([sample_x, sample_y]) as f32 * amplitude;

                amplitude *= map_data.persistence;
                frequency *= map_data.lacunarity;
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
    for y in 0..map_data.height {
        for x in 0..map_data.width {
            map[y][x] = smoothstep(min_height, max_height, map[y][x]) * map_data.map_height;
        }
    }

    map
}

struct MeshAttributes {
    indices: Vec<u32>,
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    colors: Vec<[f32; 3]>,
}

pub struct MapShape<'a> {
    map_data: &'a MapData,
}

impl<'a> MapShape<'a> {
    pub fn new(map_data: &'a MapData) -> Self {
        MapShape { map_data }
    }

    fn mesh_attributes(&self) -> MeshAttributes {
        let map = generate_noise_map(self.map_data);

        let width = map[0].len() as u32;
        let height = map.len() as u32;
        let size = (width * height) as usize;

        // initializes the attribute vectors
        let mut indices = Vec::with_capacity(((width - 1) * (height - 1) * 6) as usize);
        let mut positions = Vec::with_capacity(size);
        let mut normals = Vec::with_capacity(size);
        let mut uvs = Vec::with_capacity(size);
        let mut colors = Vec::with_capacity(size);

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

                positions.push([x as f32, map[y as usize][x as usize], y as f32]);
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

        MeshAttributes {
            indices,
            positions,
            normals,
            uvs,
            colors,
        }
    }
}

impl<'a> From<MapShape<'a>> for Mesh {
    fn from(map_shape: MapShape) -> Self {
        let MeshAttributes {
            indices,
            positions,
            normals,
            uvs,
            colors,
        } = map_shape.mesh_attributes();

        let mut mesh = if map_shape.map_data.wireframe {
            Mesh::new(PrimitiveTopology::LineList)
        } else {
            Mesh::new(PrimitiveTopology::TriangleList)
        };

        // set the attributes of the mesh
        mesh.set_indices(Some(Indices::U32(indices)));
        mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.set_attribute(MapMaterial::ATTRIBUTE_COLOR, colors);
        mesh
    }
}
