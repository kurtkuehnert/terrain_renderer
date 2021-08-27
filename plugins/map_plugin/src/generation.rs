use crate::data::{MapData, NoiseData};
use bevy::math::DVec2;
use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::pipeline::PrimitiveTopology;
use bytemuck::allocation::cast_vec;
use itertools::iproduct;
use nalgebra_glm::smoothstep;
use noise::{NoiseFn, OpenSimplex};
use rand::prelude::*;

/// Describes how a noise map is normalized to the range [0, 1].
#[allow(dead_code)]
pub enum NormalizeMode {
    /// Local is exact on a chunk by chunk bases, but introduces deviations between adjacent chunks.
    Local,
    /// Global estimates the mapping equally for all chunks,
    /// but can't guaranty that all values in range [0, 1] are reached.
    Global,
}

pub const LOD_LEVELS: usize = 7; // count of levels of detail
pub const CHUNK_SIZE: usize = 240; // side length of a map chunk, vertex count is one more
pub const HALF_CHUNK_SIZE: f32 = CHUNK_SIZE as f32 / 2.0;

const MAX_OFFSET: f64 = 1000.0; // offset value for octave noise generation

// values to estimate the maximum possible height of the noise map before normalization (global)
const AMPLITUDE_HEURISTIC: f32 = 0.7;
const HEIGHT_HEURISTIC: f32 = 0.9;

pub type NoiseMap = Vec<Vec<f32>>;

/// Generates a noise map with heights in range [0, 1] from the supplied noise data.
pub fn generate_noise_map(
    data: &NoiseData,
    chunk_coord: IVec2,
    normalize_mode: NormalizeMode,
) -> NoiseMap {
    // prepare the noise and the random number generator
    let noise = OpenSimplex::new();
    let mut rng = StdRng::seed_from_u64(data.seed);

    // determine an approximated maximum possible height difference
    // between the min an max height for global normalization
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
    // the offset the chunk based on its coordinates
    // and by half the chunk size for compensating the looping from zero to chunk size
    let chunk_offset = DVec2::new(
        (chunk_coord.x as f64 - 0.5) * CHUNK_SIZE as f64,
        (chunk_coord.y as f64 - 0.5) * CHUNK_SIZE as f64,
    );

    // determine concrete min and max values for local normalization
    let mut max_height = f32::MIN;
    let mut min_height = f32::MAX;

    // generate the noise map
    let mut map: NoiseMap = (0..=CHUNK_SIZE)
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
    pub fn new(map_data: &MapData, noise_map: &NoiseMap, lod: usize) -> Self {
        let simplification_increment = if lod == 0 { 1 } else { lod * 2 };
        let side_length = CHUNK_SIZE / simplification_increment + 1;
        let vertex_count = side_length * side_length;

        let mut indices = Vec::with_capacity((side_length - 1) * (side_length - 1) * 6);
        let mut positions = Vec::with_capacity(vertex_count);
        let mut normals = vec![Vec3::ZERO; vertex_count];
        let mut uvs = Vec::with_capacity(vertex_count);

        let side_indices = (0..=CHUNK_SIZE).step_by(simplification_increment);

        // calculate all the indices, positions, normals and uvs
        for (vertex_index, (x, y)) in iproduct!(side_indices.clone(), side_indices).enumerate() {
            // adjust the noise with the height curve, to flatten the water
            let noise_height = map_data
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
                noise_height * map_data.map_height,
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

        Self {
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

/// Generates a mesh of the map with the parameters of the map data.
/// Uses the provided noise map or initializes it.
pub fn generate_chunk(
    map_data: &MapData,
    noise_map: &mut Option<NoiseMap>,
    chunk_coord: IVec2,
    lod: usize,
) -> Mesh {
    if noise_map.is_none() {
        *noise_map = Some(generate_noise_map(
            &map_data.noise_data,
            chunk_coord,
            NormalizeMode::Global,
        ));
    }

    let noise_map = noise_map.as_ref().unwrap();

    ChunkShape::new(map_data, noise_map, lod).into()
}
