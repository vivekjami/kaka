use kaka::simhash::{SimHash, SimHashEngine};

#[test]
fn hash_consistency() {
    let engine = SimHashEngine::new(64);
    let url = "https://example.com/page";

    let h1 = engine.compute_hash_from_url(url);
    let h2 = engine.compute_hash_from_url(url);

    assert_eq!(h1, h2);
}

#[test]
fn similarity_same_domain() {
    let engine = SimHashEngine::new(64);

    let h1 = engine.compute_hash_from_url("https://example.com/page1");
    let h2 = engine.compute_hash_from_url("https://example.com/page2");

    assert!(engine.similarity(h1, h2) > 0.9);
}

#[test]
fn similarity_different_domain() {
    let engine = SimHashEngine::new(64);

    let h1 = engine.compute_hash_from_url("https://example.com/page");
    let h2 = engine.compute_hash_from_url("https://other.com/page");

    assert!(engine.similarity(h1, h2) < 0.7);
}

#[test]
fn minor_query_change_high_similarity() {
    let engine = SimHashEngine::new(64);

    let h1 = engine.compute_hash_from_url("https://example.com/article");
    let h2 = engine.compute_hash_from_url("https://example.com/article?id=1");

    assert!(engine.similarity(h1, h2) > 0.95);
}

#[test]
fn edge_cases() {
    let engine = SimHashEngine::new(64);

    engine.compute_hash_from_url("https://x.com");
    engine.compute_hash_from_url("https://example.com/");
    engine.compute_hash_from_url("https://example.com/very/long/path/with/data");
}

use proptest::prelude::*;

proptest! {
    #[test]
    fn identical_urls_same_hash(
        domain in "[a-z]{5,10}",
        path in "[a-z]{3,8}"
    ) {
        let engine = SimHashEngine::new(64);

        let url = format!("https://{}.com/{}", domain, path);

        let h1 = engine.compute_hash_from_url(&url);
        let h2 = engine.compute_hash_from_url(&url);

        prop_assert_eq!(h1, h2);
    }

    #[test]
    fn similar_urls_high_similarity(
        domain in "[a-z]{5,10}",
        page in 1u32..1000
    ) {
        let engine = SimHashEngine::new(64);

        let url1 = format!("https://{}.com/page{}", domain, page);
        let url2 = format!("https://{}.com/page{}", domain, page + 1);

        let h1 = engine.compute_hash_from_url(&url1);
        let h2 = engine.compute_hash_from_url(&url2);

        let sim = engine.similarity(h1, h2);

        prop_assert!(
            sim > 0.85,
            "Expected similarity > 0.85, got {}",
            sim
        );
    }
}
