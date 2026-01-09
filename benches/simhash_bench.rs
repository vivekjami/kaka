use criterion::{Criterion, Throughput, black_box, criterion_group, criterion_main};
use kaka::simhash::SimHashEngine;

fn simhash_compute_benchmark(c: &mut Criterion) {
    let engine = SimHashEngine::new(64);
    let urls: Vec<String> = (0..10_000)
        .map(|i| format!("https://example.com/page{}", i))
        .collect();

    let mut group = c.benchmark_group("simhash");
    group.throughput(Throughput::Elements(10_000));

    group.bench_function("compute_10k_hashes", |b| {
        b.iter(|| {
            for url in &urls {
                black_box(engine.compute_hash_from_url(url));
            }
        });
    });

    group.finish();
}

fn hamming_distance_benchmark(c: &mut Criterion) {
    let engine = SimHashEngine::new(64);
    let h1 = engine.compute_hash_from_url("https://example.com/a");
    let h2 = engine.compute_hash_from_url("https://example.com/b");

    c.bench_function("hamming_distance", |b| {
        b.iter(|| {
            black_box(SimHashEngine::hamming_distance(h1, h2));
        });
    });
}

criterion_group!(
    benches,
    simhash_compute_benchmark,
    hamming_distance_benchmark
);
criterion_main!(benches);
