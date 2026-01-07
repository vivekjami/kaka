//! Benchmarks for URL normalization.
//!
//! This measures the cost of parsing + normalization, which is
//! typically the dominant cost in crawler deduplication pipelines.

use criterion::{Criterion, Throughput, black_box, criterion_group, criterion_main};
use kaka::normalizer::UrlNormalizer;

/// Benchmark normalization of already-well-formed URLs.
/// This reflects the *best-case* hot-path scenario.
fn normalize_simple_urls(c: &mut Criterion) {
    let normalizer = UrlNormalizer::new();

    let urls: Vec<String> = (0..10_000)
        .map(|i| format!("https://example.com/page{}", i))
        .collect();

    let mut group = c.benchmark_group("normalizer_simple");
    group.throughput(Throughput::Elements(urls.len() as u64));

    group.bench_function("normalize_10k_simple", |b| {
        b.iter(|| {
            for url in &urls {
                black_box(normalizer.normalize(black_box(url)).unwrap());
            }
        });
    });

    group.finish();
}

/// Benchmark normalization of complex, real-world URLs.
/// This reflects *real crawler traffic*.
fn normalize_complex_urls(c: &mut Criterion) {
    let normalizer = UrlNormalizer::new();

    let urls: Vec<String> = (0..10_000)
        .map(|i| {
            format!(
                "HTTPS://WWW.Example.com:443/Path/../Page{}?b=2&utm_source=google&a=1#section",
                i
            )
        })
        .collect();

    let mut group = c.benchmark_group("normalizer_complex");
    group.throughput(Throughput::Elements(urls.len() as u64));

    group.bench_function("normalize_10k_complex", |b| {
        b.iter(|| {
            for url in &urls {
                black_box(normalizer.normalize(black_box(url)).unwrap());
            }
        });
    });

    group.finish();
}

/// Benchmark normalization of URLs with heavy query parameters.
/// This stresses query parsing, filtering, and sorting.
fn normalize_query_heavy_urls(c: &mut Criterion) {
    let normalizer = UrlNormalizer::new();

    let urls: Vec<String> = (0..10_000)
        .map(|i| {
            format!(
                "https://example.com/search?q=rust&b=2&a=1&utm_campaign=test{}&utm_source=google&ref=home",
                i
            )
        })
        .collect();

    let mut group = c.benchmark_group("normalizer_query_heavy");
    group.throughput(Throughput::Elements(urls.len() as u64));

    group.bench_function("normalize_10k_query_heavy", |b| {
        b.iter(|| {
            for url in &urls {
                black_box(normalizer.normalize(black_box(url)).unwrap());
            }
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    normalize_simple_urls,
    normalize_complex_urls,
    normalize_query_heavy_urls
);
criterion_main!(benches);
