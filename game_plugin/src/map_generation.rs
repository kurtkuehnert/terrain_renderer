use crate::map_data::{MapData, NoiseData};
use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::pipeline::PrimitiveTopology;
use nalgebra_glm::smoothstep;
use noise::{NoiseFn, OpenSimplex};
use rand::prelude::*;

/// Generates a noise map with heights in range [0, 1] from the supplied noise data.
fn generate_noise_map(data: &NoiseData) -> Vec<Vec<f32>> {
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
            let mut amplitude = 1.0;
            let mut frequency = 1.0;
            let mut noise_height = 0.0;

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

    // normalize the map height between 0 and 1
    for y in 0..data.height {
        for x in 0..data.width {
            map[y][x] = smoothstep(min_height, max_height, map[y][x]);
        }
    }

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
    pub fn new(map_data: &'a MapData) -> Self {
        let width = map_data.noise_data.width;
        let height = map_data.noise_data.height;
        let size = width * height;

        let mut map_shape = MapShape {
            map_data,
            indices: Vec::with_capacity((width - 1) * (height - 1) * 6),
            positions: Vec::with_capacity(size),
            normals: Vec::with_capacity(size),
            uvs: Vec::with_capacity(size),
        };

        map_shape.generate();

        map_shape
    }

    /// Generates the mesh data of the map.
    fn generate(&mut self) {
        let noise_map = generate_noise_map(&self.map_data.noise_data);

        let width = self.map_data.noise_data.width as u32;
        let height = self.map_data.noise_data.height as u32;

        let indices = &mut self.indices;
        let positions = &mut self.positions;
        let normals = &mut self.normals;
        let uvs = &mut self.uvs;

        let mut vertex_index: u32 = 0;

        // calculate all the indices
        for _ in 0..height - 1 {
            for _ in 0..width - 1 {
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

                    indices.push(vertex_index);
                    indices.push(vertex_index + width + 1);
                    indices.push(vertex_index + 1);
                }
                vertex_index += 1;
            }

            vertex_index += 1;
        }

        // calculate all the positions, normals and uvs
        for y in 0..height {
            for x in 0..width {
                let noise_height = noise_map[y as usize][x as usize];

                positions.push([x as f32, noise_height * self.map_data.map_height, y as f32]);
                normals.push([0.0, 1.0, 0.0]);
                uvs.push([x as f32 / width as f32, y as f32 / noise_height as f32]);
            }
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
