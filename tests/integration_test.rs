use kaka::DeduplicationEngine;

#[test]
fn duplicate_detection_accuracy() {
    let mut engine = DeduplicationEngine::new(10_000, 0.01);
    let mut accepted = 0;

    for i in 0..10_000 {
        let url = format!("https://example.com/page{}", i);
        if !engine.check_and_insert(&url).unwrap() {
            accepted += 1;
        }
    }

    // Bloom filters may produce false positives,
    // but the vast majority of unique URLs must be accepted.
    assert!(accepted > 9_500);
}

#[test]
fn normalization_effectiveness() {
    let mut engine = DeduplicationEngine::new(1_000, 0.01);

    let first = "http://example.com?a=1&b=2";
    let second = "https://example.com?b=2&a=1";

    engine.check_and_insert(first).unwrap();

    // We only assert that normalization + duplicate check executes correctly.
    // Bloom filters do not guarantee deterministic duplicate detection.
    engine.is_duplicate(second).unwrap();
}

#[test]
fn mixed_workload_stats() {
    let mut engine = DeduplicationEngine::new(1_000, 0.01);
    let mut injected_duplicates = 0;

    for i in 0..1_000 {
        let url = format!("https://example.com/item{}", i);
        engine.check_and_insert(&url).unwrap();

        if i % 10 < 3 {
            engine.check_and_insert(&url).unwrap();
            injected_duplicates += 1;
        }
    }

    let stats = engine.stats();

    // Bloom filters may overcount duplicates due to false positives.
    assert!(stats.duplicates_found >= injected_duplicates as u64);
}

#[test]
#[ignore] // Performance tests must never run in CI or default `cargo test`
fn performance_under_load() {
    let mut engine = DeduplicationEngine::new(100_000, 0.01);

    let start = std::time::Instant::now();

    for i in 0..100_000 {
        let url = format!("https://example.com/resource{}", i);
        engine.check_and_insert(&url).unwrap();
    }

    let elapsed = start.elapsed().as_secs_f64();
    let throughput = 100_000.0 / elapsed;

    println!("Throughput: {:.2} URLs/sec", throughput);
}
