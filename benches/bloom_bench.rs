//! Criterion benchmarks for Bloom filter performance.

use criterion::{Criterion, Throughput, black_box, criterion_group, criterion_main};
use kaka::bloom::BloomFilter;

/// Benchmark Bloom filter insertion throughput.
fn bloom_insert_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("bloom_insert");
    group.throughput(Throughput::Elements(1000));

    group.bench_function("insert_1k_urls", |b| {
        b.iter(|| {
            let mut bloom = BloomFilter::new(10_000, 0.01);
            for i in 0..1000 {
                let url = format!("https://example.com/page{}", i);
                bloom.insert(black_box(&url));
            }
        });
    });

    group.finish();
}

/// Benchmark membership check performance.
fn bloom_contains_benchmark(c: &mut Criterion) {
    let mut bloom = BloomFilter::new(10_000, 0.01);
    for i in 0..10_000 {
        bloom.insert(&format!("https://example.com/page{}", i));
    }

    c.bench_function("contains_check", |b| {
        b.iter(|| bloom.contains(black_box("https://example.com/page5000")));
    });
}

criterion_group!(benches, bloom_insert_benchmark, bloom_contains_benchmark);
criterion_main!(benches);
