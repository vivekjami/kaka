//! Bloom Filter implementation.
//!
//! This module provides a probabilistic data structure for fast set
//! membership testing. It guarantees **zero false negatives** and
//! allows a configurable false positive rate.
//!
//! Designed for high-throughput URL deduplication in large-scale
//! web crawlers and indexing systems.

use ahash::RandomState;
use bitvec::vec::BitVec;
use std::hash::Hash;

/// Bloom filter for approximate set membership testing.
///
/// # Characteristics
/// - No false negatives
/// - Configurable false positive rate
/// - Memory efficient
///
/// # Fields
/// - `bits`: Bit vector backing the filter
/// - `num_hashes`: Number of hash functions (k)
/// - `hash_builder`: Fast, randomized hash builder
/// - `items_inserted`: Count of inserted elements (n)
pub struct BloomFilter {
    bits: BitVec,
    num_hashes: u32,
    hash_builder: RandomState,
    items_inserted: u64,
}

impl BloomFilter {
    /// Create a new Bloom filter.
    ///
    /// # Arguments
    /// - `capacity`: Expected number of elements (n)
    /// - `fp_rate`: Desired false positive probability (p)
    ///
    /// # Formulae
    /// - Bit array size (m):
    ///   m = -n * ln(p) / (ln(2)^2)
    /// - Number of hash functions (k):
    ///   k = (m / n) * ln(2)
    pub fn new(capacity: usize, fp_rate: f64) -> Self {
        let ln2 = std::f64::consts::LN_2;

        let m = (-(capacity as f64) * fp_rate.ln() / (ln2 * ln2)).ceil() as usize;
        let k = ((m as f64 / capacity as f64) * ln2).ceil() as u32;

        Self {
            bits: BitVec::repeat(false, m),
            num_hashes: k,
            hash_builder: RandomState::new(),
            items_inserted: 0,
        }
    }

    /// Insert an element into the Bloom filter.
    ///
    /// Uses double hashing to simulate `k` hash functions:
    ///
    /// `position_i = (h1 + i * h2) % m`
    pub fn insert(&mut self, value: &str) {
        let (h1, h2) = self.base_hashes(value);
        let m = self.bits.len() as u64;

        for i in 0..self.num_hashes {
            let index = (h1.wrapping_add((i as u64).wrapping_mul(h2)) % m) as usize;
            self.bits.set(index, true);
        }

        self.items_inserted += 1;
    }

    /// Check whether an element is possibly in the set.
    ///
    /// Returns:
    /// - `false` if the element is **definitely not present**
    /// - `true` if the element is **possibly present**
    pub fn contains(&self, value: &str) -> bool {
        let (h1, h2) = self.base_hashes(value);
        let m = self.bits.len() as u64;

        for i in 0..self.num_hashes {
            let index = (h1.wrapping_add((i as u64).wrapping_mul(h2)) % m) as usize;
            if !self.bits[index] {
                return false;
            }
        }

        true
    }

    /// Estimate the current false positive rate.
    ///
    /// Formula:
    /// `(1 - e^(-k * n / m))^k`
    pub fn false_positive_rate(&self) -> f64 {
        let k = self.num_hashes as f64;
        let n = self.items_inserted as f64;
        let m = self.bits.len() as f64;

        (1.0 - (-k * n / m).exp()).powf(k)
    }

    /// Generate two base hashes for double hashing.
    #[inline]
    fn base_hashes<T: Hash>(&self, value: T) -> (u64, u64) {
        let h1 = self.hash_builder.hash_one(value);
        let h2 = self.hash_builder.hash_one(h1);
        (h1, h2)
    }
}
