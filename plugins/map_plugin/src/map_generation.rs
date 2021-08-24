use crate::map_data::{MapData, NoiseData};
use bevy::math::DVec2;
use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::pipeline::PrimitiveTopology;
use bytemuck::allocation::cast_vec;
use itertools::iproduct;
use nalgebra_glm::smoothstep;
use noise::{NoiseFn, OpenSimplex};
use rand::prelude::*;

enum NormalizeMode {
    Local,
    Global,
}

pub const CHUNK_SIZE: usize = 240 / 4; // side length of a map chunk, vertex count is one more
pub const HALF_CHUNK_SIZE: f32 = CHUNK_SIZE as f32 / 2.0;

const MAX_OFFSET: f64 = 1000.0; // offset value for octave noise generation

// values to estimate the maximum possible height of the noise map before normalization (global)
const AMPLITUDE_HEURISTIC: f32 = 0.7;
const HEIGHT_HEURISTIC: f32 = 0.9;

/// Generates a noise map with heights in range [0, 1] from the supplied noise data.
fn generate_noise_map(
    data: &NoiseData,
    chunk_coord: IVec2,
    normalize_mode: NormalizeMode,
) -> Vec<Vec<f32>> {
    // prepare the noise and the random number generator
    let noise = OpenSimplex::new();
    let mut rng = StdRng::seed_from_u64(data.seed);

    let mut max_possible_height = 0.0;
    let mut amplitude = 1.0;

    // generate random offsets for each octave
    let octave_offsets: Vec<DVec2> = (0..data.octaves)
        .map(|_| {
            max_possible_height += amplitude;
            amplitude *= data.persistence * AMPLITUDE_HEURISTIC;
            DVec2::new(
                rng.gen_range(-MAX_OFFSET..=MAX_OFFSET),
                rng.gen_range(-MAX_OFFSET..=MAX_OFFSET),
            )
        })
        .collect();

    max_possible_height *= HEIGHT_HEURISTIC;

    // approximated spread around zero
    let spread = max_possible_height / 2.0;

    // sanity check the scale
    let scale = data.scale.max(f64::EPSILON);
    // offset by half the chunk size for scaling by the center
    let half_size = (CHUNK_SIZE + 1) as f64 / 2.0;
    // the offset of the chunk
    let chunk_offset = DVec2::new(
        (chunk_coord.x * (CHUNK_SIZE as i32 + 1)) as f64 - half_size,
        (chunk_coord.y * (CHUNK_SIZE as i32 + 1)) as f64 - half_size,
    );

    let mut max_height = f32::MIN;
    let mut min_height = f32::MAX;

    // generate the noise map
    let mut map: Vec<Vec<f32>> = (0..=CHUNK_SIZE)
        .map(|y| {
            (0..=CHUNK_SIZE)
                .map(|x| {
                    let mut noise_height = 0.0;
                    let mut amplitude = 1.0;
                    let mut frequency = 1.0;

                    // sum up the height at the position for all octaves
                    for &octave_offset in &octave_offsets {
                        let sample = (DVec2::new(x as f64, y as f64) + chunk_offset) / scale
                            * frequency
                            + octave_offset;
                        noise_height += noise.get([sample.x, sample.y]) as f32 * amplitude;

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
        row.iter_mut().for_each(|height| {
            *height = match normalize_mode {
                NormalizeMode::Local => smoothstep(min_height, max_height, *height),
                NormalizeMode::Global => smoothstep(-spread, spread, *height / max_possible_height),
            }
        })
    });

    map
}

/// Intermediate chunk representation storing the entire mesh data of a chunk.
pub struct ChunkShape {
    indices: Vec<u32>,
    positions: Vec<Vec3>,
    normals: Vec<Vec3>,
    uvs: Vec<[f32; 2]>,
}

impl ChunkShape {
    pub fn new(data: &MapData, chunk_coord: IVec2) -> Self {
        let simplification_increment = if data.level_of_detail == 0 {
            1
        } else {
            data.level_of_detail * 2
        };
        let side_length = CHUNK_SIZE / simplification_increment + 1;
        let vertex_count = side_length * side_length;

        let mut indices = Vec::with_capacity((side_length - 1) * (side_length - 1) * 6);
        let mut positions = Vec::with_capacity(vertex_count);
        let mut normals = vec![Vec3::ZERO; vertex_count];
        let mut uvs = Vec::with_capacity(vertex_count);

        let noise_map = generate_noise_map(&data.noise_data, chunk_coord, NormalizeMode::Global);

        let side_indices = (0..=CHUNK_SIZE).step_by(simplification_increment);

        // calculate all the indices, positions, normals and uvs
        for (vertex_index, (x, y)) in iproduct!(side_indices.clone(), side_indices).enumerate() {
            // adjust the noise with the height curve, to flatten the water
            let noise_height = data
                .height_curve
                .evaluate(noise_map[y as usize][x as usize]);

            // add the indices
            if x < CHUNK_SIZE && y < CHUNK_SIZE {
                indices.push(vertex_index as u32);
                indices.push(vertex_index as u32 + 1);
                indices.push((vertex_index + side_length) as u32 + 1);
                indices.push(vertex_index as u32);
                indices.push((vertex_index + side_length) as u32 + 1);
                indices.push((vertex_index + side_length) as u32);
            }

            // calculate and add the position, normal and uv for the vertex to the shape
            positions.push(Vec3::new(
                x as f32 - HALF_CHUNK_SIZE,
                noise_height * data.map_height,
                y as f32 - HALF_CHUNK_SIZE,
            ));
            uvs.push([0.0, 0.0]);
        }

        // calculate the normals in the center of each triangle and combine them in each vertex
        indices.chunks_exact(3).for_each(|triangle| {
            let i1 = triangle[0] as usize;
            let i2 = triangle[1] as usize;
            let i3 = triangle[2] as usize;

            let pos1 = positions[i1];
            let pos2 = positions[i2];
            let pos3 = positions[i3];

            let vec12 = pos2 - pos1;
            let vec13 = pos3 - pos1;

            let normal = vec13.cross(vec12);

            normals[i1] += normal;
            normals[i2] += normal;
            normals[i3] += normal;
        });

        // normalize the normals
        normals
            .iter_mut()
            .for_each(|normal| *normal = normal.normalize());

        ChunkShape {
            indices,
            positions,
            normals,
            uvs,
        }
    }
}

impl From<ChunkShape> for Mesh {
    fn from(map_shape: ChunkShape) -> Mesh {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

        // set the attributes of the mesh
        mesh.set_indices(Some(Indices::U32(map_shape.indices)));
        mesh.set_attribute(
            Mesh::ATTRIBUTE_POSITION,
            cast_vec::<Vec3, [f32; 3]>(map_shape.positions),
        );
        mesh.set_attribute(
            Mesh::ATTRIBUTE_NORMAL,
            cast_vec::<Vec3, [f32; 3]>(map_shape.normals),
        );
        mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, map_shape.uvs);

        mesh
    }
}
