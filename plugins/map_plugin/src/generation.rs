use crate::data::{MapTopologyData, NoiseData};
use bevy::{
    math::{DVec2, IVec2},
    prelude::*,
    render::texture::{Extent3d, TextureDimension, TextureFormat},
};
use itertools::iproduct;
use nalgebra_glm::smoothstep;
use noise::{NoiseFn, OpenSimplex};
use rand::{prelude::StdRng, Rng, SeedableRng};

/// Offset value for octave noise generation.
const MAX_OFFSET: f64 = 1000.0;

pub type NoiseMap = Vec<f32>;

/// Generates a noise map with heights in range [0, 1] from the supplied noise data.
pub fn generate_noise_map(data: &NoiseData, map_size: IVec2) -> NoiseMap {
    // prepare the noise and the random number generator
    let noise = OpenSimplex::new();
    let mut rng = StdRng::seed_from_u64(data.seed);

    // generate random offsets for each octave
    let octave_offsets: Vec<DVec2> = (0..data.octaves)
        .map(|_| {
            DVec2::new(
                rng.gen_range(-MAX_OFFSET..=MAX_OFFSET),
                rng.gen_range(-MAX_OFFSET..=MAX_OFFSET),
            )
        })
        .collect();

    // sanity check the scale
    let scale = data.scale.max(f64::EPSILON);

    let mut max_height = f32::MIN;
    let mut min_height = f32::MAX;

    // generate the noise map
    let mut map: NoiseMap = iproduct!((0..map_size.y), (0..map_size.x))
        .map(|(y, x)| {
            let mut noise_height = 0.0;
            let mut amplitude = 1.0;
            let mut frequency = 1.0;

            // sum up the height at the position for all octaves
            for &octave_offset in &octave_offsets {
                let sample = DVec2::new(x as f64, y as f64) / scale * frequency + octave_offset;
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
        .collect();

    // normalize the map height between 0 and 1
    map.iter_mut()
        .for_each(|height| *height = smoothstep(min_height, max_height, *height));

    map
}

pub fn generate_height_map(topology_data: MapTopologyData) -> Texture {
    let map = generate_noise_map(&topology_data.noise_data, topology_data.map_size.as_ivec2());

    let data = map
        .iter()
        .map(|height| {
            let height = (65536.0 * height) as u16;
            [(height / 256) as u8, (height % 256) as u8]
        })
        .flatten()
        .collect();

    Texture::new(
        Extent3d::new(
            topology_data.map_size.x as u32,
            topology_data.map_size.y as u32,
            1,
        ),
        TextureDimension::D2,
        data,
        TextureFormat::Rg8Unorm,
    )
}
