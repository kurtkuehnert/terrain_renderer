use criterion::{black_box, criterion_group, criterion_main, Criterion};
use game_plugin::map::generate_noise_map;

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("map generation: ", |b| {
        b.iter(|| generate_noise_map(black_box(100), black_box(100), black_box(1.), black_box(1)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
