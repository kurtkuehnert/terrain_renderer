use bevy::math::IVec2;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use map_plugin::{
    data::{MapData, NoiseData},
    generation::{generate_chunk, generate_noise_map, LOD_LEVELS},
};

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("height map generation: ", |b| {
        b.iter(|| {
            generate_noise_map(
                black_box(&NoiseData::default()),
                black_box(IVec2::default()),
            )
        })
    });

    c.bench_function("chunk generation without height map", |b| {
        b.iter(|| {
            generate_chunk(
                black_box(&MapData::default()),
                black_box(&mut None),
                black_box(IVec2::default()),
                black_box(LOD_LEVELS - 1),
            )
        })
    });

    let noise_map = &mut Some(generate_noise_map(&NoiseData::default(), IVec2::default()));

    c.bench_function("chunk generation with height map", |b| {
        b.iter(|| {
            generate_chunk(
                black_box(&MapData::default()),
                black_box(noise_map),
                black_box(IVec2::default()),
                black_box(LOD_LEVELS - 1),
            )
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
