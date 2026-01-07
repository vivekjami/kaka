
pub mod simhash;
pub mod lshbloom;
pub mod engine;
///! Public library interface for Kāka.
///!
///! This module wires together the Bloom filter and URL normalizer
///! into a single deduplication engine.

pub mod bloom;
pub mod normalizer;

use bloom::BloomFilter;
use normalizer::UrlNormalizer;

use std::sync::atomic::{AtomicU64, Ordering};

pub use bloom::BloomFilter as _;
pub use normalizer::UrlNormalizer as _;

/// Deduplication engine combining normalization and Bloom filtering.
pub struct DeduplicationEngine {
    bloom: BloomFilter,
    normalizer: UrlNormalizer,
    stats: Stats,
}

/// Internal statistics for observability and testing.
struct Stats {
    total_checked: AtomicU64,
    duplicates_found: AtomicU64,
    urls_inserted: AtomicU64,
}

impl DeduplicationEngine {
    /// Create a new deduplication engine.
    ///
    /// # Arguments
    /// - `capacity`: Expected number of unique URLs
    /// - `fp_rate`: Desired Bloom filter false-positive rate
    pub fn new(capacity: usize, fp_rate: f64) -> Self {
        DeduplicationEngine {
            bloom: BloomFilter::new(capacity, fp_rate),
            normalizer: UrlNormalizer::new(),
            stats: Stats {
                total_checked: AtomicU64::new(0),
                duplicates_found: AtomicU64::new(0),
                urls_inserted: AtomicU64::new(0),
            },
        }
    }

    /// Normalize, check, and insert a URL.
    ///
    /// # Returns
    /// - `Ok(false)` → URL is new
    /// - `Ok(true)` → URL is a duplicate
    pub fn check_and_insert(
        &mut self,
        url: &str,
    ) -> Result<bool, url::ParseError> {
        self.stats.total_checked.fetch_add(1, Ordering::Relaxed);

        let normalized = self.normalizer.normalize(url)?;

        if self.bloom.contains(&normalized) {
            self.stats
                .duplicates_found
                .fetch_add(1, Ordering::Relaxed);
            Ok(true)
        } else {
            self.bloom.insert(&normalized);
            self.stats
                .urls_inserted
                .fetch_add(1, Ordering::Relaxed);
            Ok(false)
        }
    }

    /// Check whether a URL is a duplicate without inserting it.
    pub fn is_duplicate(
        &self,
        url: &str,
    ) -> Result<bool, url::ParseError> {
        let normalized = self.normalizer.normalize(url)?;
        Ok(self.bloom.contains(&normalized))
    }

    /// Access internal statistics (read-only).
    pub fn stats(&self) -> EngineStatsSnapshot {
        EngineStatsSnapshot {
            total_checked: self.stats.total_checked.load(Ordering::Relaxed),
            duplicates_found: self.stats.duplicates_found.load(Ordering::Relaxed),
            urls_inserted: self.stats.urls_inserted.load(Ordering::Relaxed),
        }
    }
}

/// Immutable snapshot of engine statistics.
pub struct EngineStatsSnapshot {
    pub total_checked: u64,
    pub duplicates_found: u64,
    pub urls_inserted: u64,
}
