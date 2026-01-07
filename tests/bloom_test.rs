//! Unit and property-based tests for the Bloom filter.

use kaka::bloom::BloomFilter;
use proptest::prelude::*;

#[test]
fn basic_insertion_and_lookup() {
    let mut bloom = BloomFilter::new(1000, 0.01);

    for i in 0..1000 {
        bloom.insert(&format!("https://example.com/{}", i));
    }

    for i in 0..1000 {
        assert!(bloom.contains(&format!("https://example.com/{}", i)));
    }
}

#[test]
fn false_positive_rate_within_expected_bounds() {
    let mut bloom = BloomFilter::new(10_000, 0.01);

    for i in 0..10_000 {
        bloom.insert(&format!("https://example.com/{}", i));
    }

    let trials = 100_000;
    let mut false_positives = 0;

    for i in 10_000..(10_000 + trials) {
        if bloom.contains(&format!("https://example.com/{}", i)) {
            false_positives += 1;
        }
    }

    let measured_fp = false_positives as f64 / trials as f64;
    assert!(measured_fp <= 0.011);
}

#[test]
fn edge_cases_are_handled() {
    let mut bloom = BloomFilter::new(100, 0.01);

    bloom.insert("");
    bloom.insert(&"a".repeat(3000));
    bloom.insert("https://例え.テスト");

    assert!(bloom.contains(""));
    assert!(bloom.contains(&"a".repeat(3000)));
    assert!(bloom.contains("https://例え.テスト"));
}

proptest! {
    #[test]
    fn no_false_negatives(urls in prop::collection::vec(".*", 1..1000)) {
        let mut bloom = BloomFilter::new(1000, 0.01);

        for url in &urls {
            bloom.insert(url);
        }

        for url in &urls {
            assert!(
                bloom.contains(url),
                "False negative detected for: {}",
                url
            );
        }
    }
}
