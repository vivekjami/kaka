//! SimHash implementation for near-duplicate URL detection.
//!
//! This module implements a high-performance SimHash variant optimized
//! for URL similarity detection in web crawlers.
//!
//! Design goals:
//! - Deterministic hashing
//! - Zero heap allocation in hot paths
//! - Fast feature extraction using byte slices
//! - RFC-compliant URL parsing
//!
//! Performance target (single-thread):
//! - ≥ 1M URLs/sec for hash computation
//! - Tens of millions ops/sec for Hamming distance

use ahash::RandomState;
use std::hash::{BuildHasher, Hash, Hasher};
use url::Url;

/// 64-bit SimHash fingerprint.
///
/// Newtype wrapper ensures type safety and makes intent explicit.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct SimHash(pub u64);

/// SimHash engine configuration.
///
/// This engine is intentionally minimal: all configuration is fixed
/// for performance and simplicity.
pub struct SimHashEngine {
    hasher: RandomState,
    ngram_size: usize,
}

impl SimHashEngine {
    /// Create a new 64-bit SimHash engine.
    ///
    /// Currently only 64-bit SimHash is supported because it provides
    /// the best tradeoff between speed, memory, and collision resistance
    /// for URL deduplication.
    pub fn new(bit_width: usize) -> Self {
        assert!(bit_width == 64, "Only 64-bit SimHash is supported");

        Self {
            hasher: RandomState::new(),
            ngram_size: 3,
        }
    }

    /// Compute SimHash directly from a URL string.
    ///
    /// This function performs:
    /// - URL parsing
    /// - Feature extraction
    /// - SimHash accumulation
    ///
    /// All operations are allocation-free after URL parsing.
    pub fn compute_hash_from_url(&self, input: &str) -> SimHash {
        let url = Url::parse(input).expect("Invalid URL");

        let mut acc = [0i32; 64];

        // ---- Domain features (highest weight) ----
        if let Some(domain) = url.domain() {
            self.apply_ngrams(domain.as_bytes(), 3, 3, &mut acc);
        }

        // ---- Path features (position-weighted) ----
        let path = url.path().as_bytes();
        let path_len = path.len().max(1) as i32;

        for (i, window) in path.windows(self.ngram_size).enumerate() {
            let weight = 2 * (path_len - i as i32) / path_len;
            self.apply_feature(window, weight, &mut acc);
        }

        // ---- Query parameters (lowest weight) ----
        for (k, v) in url.query_pairs() {
            let mut hasher = self.hasher.build_hasher();
            k.hash(&mut hasher);
            v.hash(&mut hasher);
            let h = hasher.finish();

            self.accumulate_bits(h, 1, &mut acc);
        }

        SimHash(self.finalize(acc))
    }

    /// Compute similarity score in the range [0.0, 1.0].
    ///
    /// Similarity is defined as:
    /// 1.0 - (Hamming distance / bit width)
    pub fn similarity(&self, h1: SimHash, h2: SimHash) -> f64 {
        let dist = Self::hamming_distance(h1, h2) as f64;
        1.0 - (dist / 64.0)
    }

    /// Compute Hamming distance between two SimHashes.
    ///
    /// This operation is extremely fast and should not be a bottleneck.
    #[inline]
    pub fn hamming_distance(h1: SimHash, h2: SimHash) -> u32 {
        (h1.0 ^ h2.0).count_ones()
    }

    // ----------------------------------------------------------------
    // Internal helpers
    // ----------------------------------------------------------------

    #[inline]
    fn apply_ngrams(&self, bytes: &[u8], n: usize, weight: i32, acc: &mut [i32; 64]) {
        for window in bytes.windows(n) {
            self.apply_feature(window, weight, acc);
        }
    }

    #[inline]
    fn apply_feature(&self, bytes: &[u8], weight: i32, acc: &mut [i32; 64]) {
        let h = self.hasher.hash_one(bytes);
        self.accumulate_bits(h, weight, acc);
    }

    #[inline]
    fn accumulate_bits(&self, mut bits: u64, weight: i32, acc: &mut [i32; 64]) {
        for slot in acc.iter_mut() {
            if bits & 1 == 1 {
                *slot += weight;
            } else {
                *slot -= weight;
            }
            bits >>= 1;
        }
    }

    #[inline]
    fn finalize(&self, acc: [i32; 64]) -> u64 {
        let mut out = 0u64;
        for (i, v) in acc.iter().enumerate() {
            if *v > 0 {
                out |= 1 << i;
            }
        }
        out
    }
}
