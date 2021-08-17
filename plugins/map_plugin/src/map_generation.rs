use crate::map_data::{MapData, NoiseData};
use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::pipeline::PrimitiveTopology;
use itertools::Itertools;
use nalgebra_glm::smoothstep;
use noise::{NoiseFn, OpenSimplex};
use rand::prelude::*;

const MAX_OFFSET: f64 = 1000.0;
const CHUNK_SIZE: usize = 241;

/// Generates a noise map with heights in range [0, 1] from the supplied noise data.
fn generate_noise_map(data: &NoiseData) -> Vec<Vec<f32>> {
    // prepare the noise and the random number generator
    let noise = OpenSimplex::new();
    let mut rng = StdRng::seed_from_u64(data.seed);

    // generate random offsets for each octave
    let octave_offsets: Vec<(f64, f64)> = (0..data.octaves)
        .map(|_| {
            (
                rng.gen_range(-MAX_OFFSET..MAX_OFFSET),
                rng.gen_range(-MAX_OFFSET..MAX_OFFSET),
            )
        })
        .collect();

    let scale = data.scale.max(0.001); // sanity check the scale
    let half_size = CHUNK_SIZE as f64 / 2.0;
    let mut max_height = f32::MIN;
    let mut min_height = f32::MAX;

    // generate the noise map
    let mut map: Vec<Vec<f32>> = (0..CHUNK_SIZE)
        .map(|y| {
            (0..CHUNK_SIZE)
                .map(|x| {
                    let mut noise_height = 0.0;
                    let mut amplitude = 1.0;
                    let mut frequency = 1.0;

                    // sum up the height at the position for all octaves
                    for &(offset_x, offset_y) in &octave_offsets {
                        let sample_x = (x as f64 - half_size) / scale * frequency + offset_x;
                        let sample_y = (y as f64 - half_size) / scale * frequency + offset_y;
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

                    noise_height
                })
                .collect()
        })
        .collect();

    // normalize the map height between 0 and 1
    map.iter_mut().for_each(|row| {
        row.iter_mut()
            .for_each(|height| *height = smoothstep(min_height, max_height, *height))
    });

    map
}

/// Intermediate map representation storing the entire mesh data of a map.
pub struct MapShape<'a> {
    map_data: &'a MapData,
    indices: Vec<u32>,
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
}

impl<'a> MapShape<'a> {
    pub fn new(data: &'a MapData) -> Self {
        let simplification_increment = if data.level_of_detail == 0 {
            1
        } else {
            data.level_of_detail * 2
        };
        let side_length = (CHUNK_SIZE - 1) / simplification_increment + 1;
        let vertex_count = side_length * side_length;

        let mut map_shape = MapShape {
            map_data: data,
            indices: Vec::with_capacity((side_length - 1) * (side_length - 1) * 6),
            positions: Vec::with_capacity(vertex_count),
            normals: Vec::with_capacity(vertex_count),
            uvs: Vec::with_capacity(vertex_count),
        };

        let noise_map = generate_noise_map(&data.noise_data);

        let side_indices = (0..CHUNK_SIZE).step_by(simplification_increment);

        // calculate all the indices, positions, normals and uvs
        for (vertex_index, (x, y)) in side_indices
            .clone()
            .cartesian_product(side_indices)
            .enumerate()
        {
            // adjust the noise with the height curve, to flatten the water
            let noise_height = data
                .height_curve
                .evaluate(noise_map[y as usize][x as usize]);

            // add the indices
            if x < CHUNK_SIZE - 1 && y < CHUNK_SIZE - 1 {
                map_shape.add_quad(
                    vertex_index as u32,
                    vertex_index as u32 + 1,
                    (vertex_index + side_length) as u32 + 1,
                    (vertex_index + side_length) as u32,
                );
            }

            // calculate and add the position, normal and uv for the vertex to the shape
            map_shape
                .positions
                .push([x as f32, noise_height * data.map_height, y as f32]);
            map_shape.normals.push([0.0, 1.0, 0.0]);
            map_shape.uvs.push([0.0, 1.0]);
        }

        map_shape
    }

    /// Adds all of the indices for a quad to the shape.
    fn add_quad(&mut self, i1: u32, i2: u32, i3: u32, i4: u32) {
        if self.map_data.wireframe {
            self.indices.push(i4);
            self.indices.push(i3);
            self.indices.push(i1);
            self.indices.push(i4);
            self.indices.push(i1);
            self.indices.push(i2);
            self.indices.push(i2);
            self.indices.push(i3);
            self.indices.push(i3);
            self.indices.push(i1);
        } else {
            self.indices.push(i1);
            self.indices.push(i2);
            self.indices.push(i3);

            self.indices.push(i1);
            self.indices.push(i3);
            self.indices.push(i4);
        }
    }
}

impl<'a> From<MapShape<'a>> for Mesh {
    fn from(map_shape: MapShape) -> Self {
        // select type of mesh: wireframe or filled
        let mut mesh = if map_shape.map_data.wireframe {
            Mesh::new(PrimitiveTopology::LineList)
        } else {
            Mesh::new(PrimitiveTopology::TriangleList)
        };

        // set the attributes of the mesh
        mesh.set_indices(Some(Indices::U32(map_shape.indices)));
        mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, map_shape.positions);
        mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, map_shape.normals);
        mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, map_shape.uvs);

        mesh
    }
}
