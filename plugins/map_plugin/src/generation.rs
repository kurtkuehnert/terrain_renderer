use crate::data::{MapData, NoiseData};
use bevy::{
    math::{DVec2, IVec2, Vec2, Vec3},
    prelude::Mesh,
    render::{mesh::Indices, pipeline::PrimitiveTopology},
};
use bytemuck::allocation::cast_vec;
use itertools::iproduct;
use nalgebra_glm::smoothstep;
use noise::{NoiseFn, OpenSimplex};
use rand::{prelude::StdRng, Rng, SeedableRng};

/// Count of levels of detail.
pub const LOD_LEVELS: usize = 7;
/// The size of the border. Equals twice the lod levels
/// to have an additional vertex for every simplification increment.
pub const BORDER_SIZE: usize = 2 * LOD_LEVELS;
/// Side length of a chunk. The vertex count is one more.
pub const CHUNK_SIZE: usize = 240;
/// Side length of a chunk with its border for normal seam fixing.
pub const BORDERD_SIZE: usize = CHUNK_SIZE + 2 * BORDER_SIZE;
pub const HALF_CHUNK_SIZE: f32 = CHUNK_SIZE as f32 / 2.0;

/// Offset value for octave noise generation.
const MAX_OFFSET: f64 = 1000.0;

// values to estimate the maximum possible height of the noise map before normalization (global)
const AMPLITUDE_HEURISTIC: f32 = 0.7;
const HEIGHT_HEURISTIC: f32 = 0.9;

pub type NoiseMap = Vec<Vec<f32>>;

/// Generates a noise map with heights in range [0, 1] from the supplied noise data.
pub fn generate_noise_map(data: &NoiseData, chunk_coord: IVec2) -> NoiseMap {
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

    // generate the noise map
    let mut map: NoiseMap = (0..=BORDERD_SIZE)
        .map(|y| {
            (0..=BORDERD_SIZE)
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

                    noise_height
                })
                .collect()
        })
        .collect();

    // normalize the map height between 0 and 1
    map.iter_mut().for_each(|row| {
        row.iter_mut().for_each(|height| {
            *height = smoothstep(-spread, spread, *height / max_possible_height);
        })
    });

    map
}

/// Vertex index mapping to the its corresponding position, uv and normal.
type Index = (IndexType, usize);

/// Signifys whether a vertex is part of the mesh or the border surrounding it.
/// Verticies of the border are only used to adjust the normals at the edge of the chunks.
#[derive(PartialEq, Eq, Clone, Copy)]
enum IndexType {
    Mesh,
    Border,
}

/// Intermediate chunk representation storing the entire mesh data of a chunk.
pub struct ChunkShape {
    mesh_indices: Vec<usize>,
    mesh_positions: Vec<Vec3>,
    normals: Vec<Vec3>,
    uvs: Vec<Vec2>,
    border_indices: Vec<Index>,
    border_positions: Vec<Vec3>,
}

impl ChunkShape {
    #[allow(clippy::ptr_arg)]
    pub fn new(map_data: &MapData, noise_map: &NoiseMap, lod: usize) -> Self {
        // every nth vertex is sampled from the noise map for the mesh
        let simplification_increment = if lod == 0 { 1 } else { lod * 2 };

        // vertex count per side of the mesh
        let side_vertex_count = CHUNK_SIZE / simplification_increment + 1;

        // vertex count of the mesh
        let mesh_vertex_count = side_vertex_count * side_vertex_count;

        let mut shape = Self {
            mesh_indices: Vec::with_capacity((side_vertex_count - 1) * (side_vertex_count - 1) * 6),
            mesh_positions: Vec::with_capacity(mesh_vertex_count),
            normals: vec![Vec3::ZERO; mesh_vertex_count],
            uvs: Vec::with_capacity(mesh_vertex_count),
            border_indices: Vec::with_capacity(side_vertex_count * 24),
            border_positions: Vec::with_capacity(side_vertex_count * 4 + 4),
        };

        shape.calculate_verticies(
            map_data,
            noise_map,
            simplification_increment,
            side_vertex_count,
        );

        shape.calculate_normals();

        shape
    }

    /// Calcualtes the indices, positions and uvs of the mesh.
    #[allow(clippy::ptr_arg)]
    fn calculate_verticies(
        &mut self,
        map_data: &MapData,
        noise_map: &NoiseMap,
        simplification_increment: usize,
        side_vertex_count: usize,
    ) {
        // the first index sampled from the noise map
        let first = BORDER_SIZE - simplification_increment;
        // the last index sampled from the noise map
        let last = BORDER_SIZE + CHUNK_SIZE + simplification_increment;

        // iterator over every nth (simplyfication increment) value of a side of the noise map
        let side_indices = (first..=last).step_by(simplification_increment);

        // precompute a map storing the indices (mesh or border) for every vertex
        let mut mesh_index = 0;
        let mut border_index = 0;
        let indices_map: Vec<Index> = iproduct!(side_indices.clone(), side_indices.clone())
            .into_iter()
            .map(|(x, y)| {
                let is_border = x == first || y == first || x == last || y == last;

                if is_border {
                    let index = border_index;
                    border_index += 1;
                    (IndexType::Border, index)
                } else {
                    let index = mesh_index;
                    mesh_index += 1;
                    (IndexType::Mesh, index)
                }
            })
            .collect();

        // compute all the indices, positions, normals and uvs
        for (index, (x, y)) in iproduct!(side_indices.clone(), side_indices).enumerate() {
            // determin the proper index of the vertex
            let index_a = indices_map[index];

            // position percentage relativ the the mesh (0, 0) -> bottom left, (1, 1) -> top right
            let percent = Vec2::new(
                (x as f32 - BORDER_SIZE as f32) / CHUNK_SIZE as f32,
                (y as f32 - BORDER_SIZE as f32) / CHUNK_SIZE as f32,
            );

            // Todo: change noise map to height map
            // sample the height by adjusting the noise with the height curve, to flatten the water
            let noise_height = map_data.height_curve.evaluate(noise_map[y][x]);

            // calculate the vertex position, while mapping the center to (0,0)
            let position = Vec3::new(
                (percent.x - 0.5) * CHUNK_SIZE as f32,
                noise_height * map_data.map_height,
                (percent.y - 0.5) * CHUNK_SIZE as f32,
            );

            // insert the position and optionally the uv into the corresponding buffers
            match index_a.0 {
                IndexType::Mesh => {
                    self.mesh_positions.push(position);
                    self.uvs.push(percent);
                }
                IndexType::Border => self.border_positions.push(position),
            };

            // add the indices of the two triangles
            if x < last && y < last {
                // determin the remaining three vertex indicies
                let index_b = indices_map[index + 1];
                let index_c = indices_map[index + side_vertex_count + 2 + 1];
                let index_d = indices_map[index + side_vertex_count + 2];

                self.add_triangle(index_a, index_b, index_c);
                self.add_triangle(index_a, index_c, index_d);
            }
        }
    }

    /// Calculate the normals in the center of each triangle and combine them in each vertex.
    /// Uses the borderd positions to correctly match normals between adjecent normals.
    fn calculate_normals(&mut self) {
        let Self {
            mesh_indices,
            mesh_positions,
            border_indices,
            border_positions,
            normals,
            ..
        } = self;

        // calculate the amounts contributed by the mesh triangles
        mesh_indices.chunks_exact(3).for_each(|triangle| {
            let index_a = triangle[0];
            let index_b = triangle[1];
            let index_c = triangle[2];

            let normal = Self::calculate_surface_normal(
                mesh_positions[index_a],
                mesh_positions[index_b],
                mesh_positions[index_c],
            );

            normals[index_a] += normal;
            normals[index_b] += normal;
            normals[index_c] += normal;
        });

        // calculate the amounts contributed by the border triangles
        border_indices.chunks_exact(3).for_each(|triangle| {
            let index_a = triangle[0];
            let index_b = triangle[1];
            let index_c = triangle[2];

            let pos_a = match index_a.0 {
                IndexType::Mesh => mesh_positions[index_a.1],
                IndexType::Border => border_positions[index_a.1],
            };
            let pos_b = match index_b.0 {
                IndexType::Mesh => mesh_positions[index_b.1],
                IndexType::Border => border_positions[index_b.1],
            };
            let pos_c = match index_c.0 {
                IndexType::Mesh => mesh_positions[index_c.1],
                IndexType::Border => border_positions[index_c.1],
            };

            let normal = Self::calculate_surface_normal(pos_a, pos_b, pos_c);

            // normals of border triangles only contribute to normals that are part of the mesh
            if index_a.0 == IndexType::Mesh {
                normals[index_a.1] += normal;
            }
            if index_b.0 == IndexType::Mesh {
                normals[index_b.1] += normal;
            }
            if index_c.0 == IndexType::Mesh {
                normals[index_c.1] += normal;
            }
        });

        // normalize the normals
        self.normals
            .iter_mut()
            .for_each(|normal| *normal = normal.normalize());
    }

    /// Calculates the center surafce normal of a triangle from three points.
    fn calculate_surface_normal(pos_a: Vec3, pos_b: Vec3, pos_c: Vec3) -> Vec3 {
        let vec_ab = pos_b - pos_a;
        let vec_ac = pos_c - pos_a;

        vec_ac.cross(vec_ab)
    }

    /// Adds the three indicies into the corresponding buffer.
    fn add_triangle(&mut self, index_a: Index, index_b: Index, index_c: Index) {
        if index_a.0 == IndexType::Border
            || index_b.0 == IndexType::Border
            || index_c.0 == IndexType::Border
        {
            self.border_indices.push(index_a);
            self.border_indices.push(index_b);
            self.border_indices.push(index_c);
        } else {
            self.mesh_indices.push(index_a.1);
            self.mesh_indices.push(index_b.1);
            self.mesh_indices.push(index_c.1);
        }
    }
}

impl From<ChunkShape> for Mesh {
    fn from(map_shape: ChunkShape) -> Mesh {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

        // set the attributes of the mesh
        mesh.set_indices(Some(Indices::U32(
            map_shape
                .mesh_indices
                .into_iter()
                .map(|index| index as u32)
                .collect(),
        )));
        mesh.set_attribute(
            Mesh::ATTRIBUTE_POSITION,
            cast_vec::<Vec3, [f32; 3]>(map_shape.mesh_positions),
        );
        mesh.set_attribute(
            Mesh::ATTRIBUTE_NORMAL,
            cast_vec::<Vec3, [f32; 3]>(map_shape.normals),
        );
        mesh.set_attribute(
            Mesh::ATTRIBUTE_UV_0,
            cast_vec::<Vec2, [f32; 2]>(map_shape.uvs),
        );

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
        *noise_map = Some(generate_noise_map(&map_data.noise_data, chunk_coord));
    }

    let noise_map = noise_map.as_ref().unwrap();

    ChunkShape::new(map_data, noise_map, lod).into()
}
