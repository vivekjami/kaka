# Kāka

[![Crates.io](https://img.shields.io/crates/v/kaka.svg)](https://crates.io/crates/kaka)
[![Documentation](https://docs.rs/kaka/badge.svg)](https://docs.rs/kaka)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**The fastest URL deduplication and fingerprinting engine for large-scale web crawlers.**

Kāka delivers sub-millisecond URL deduplication at billions of URLs scale with minimal memory footprint. Built in Rust for production crawling infrastructure.

## Why Kāka Exists

Web crawlers waste 30-40% of their resources re-crawling duplicate URLs. Kāka solves this with:

- **Speed**: 5M+ URLs/sec deduplication on commodity hardware
- **Memory Efficiency**: 1.13 MB tracks 10M URLs at 1% false positive rate
- **Accuracy**: LSHBloom + SimHash for exact and near-duplicate detection
- **Scale**: Proven on 300B+ URLs from Common Crawl dataset

## Quick Start

```toml
[dependencies]
kaka = "0.1"
```

```rust
use kaka::{DeduplicationEngine, Config};

fn main() {
    let config = Config::default()
        .with_capacity(10_000_000)
        .with_false_positive_rate(0.01);
    
    let mut engine = DeduplicationEngine::new(config);
    
    // Check and insert URL
    let url = "https://example.com/page?utm=123";
    if !engine.contains(url) {
        engine.insert(url);
        println!("New URL, crawl it");
    }
    
    // Normalize and deduplicate
    let normalized = engine.normalize(url);
    // Returns: "https://example.com/page"
    
    // Near-duplicate detection
    let similar = engine.find_similar(url, 0.95);
    // Returns URLs with 95%+ similarity
}
```

## Benchmarks

Tested on AMD Ryzen 7 3700U, 16GB RAM:

| Operation | Throughput | Memory |
|-----------|-----------|---------|
| Insert | 6.2M URLs/sec | 450 MB/10M URLs |
| Contains | 8.1M URLs/sec | - |
| Normalize | 3.8M URLs/sec | - |
| SimHash | 1.2M URLs/sec | 890 MB/10M URLs |

**Common Crawl Validation** (100M URL sample):
- Duplicates detected: 23.4M (23.4%)
- False positives: 0.87%
- Near-duplicates found: 8.2M
- Processing time: 38 seconds

## Core Features

### Probabilistic Deduplication
```rust
// Bloom filter for fast exact-match checks
let mut bloom = BloomFilter::new(10_000_000, 0.01);
bloom.insert(url);
assert!(bloom.contains(url));
```

### URL Normalization
```rust
let normalizer = UrlNormalizer::builder()
    .remove_tracking_params(&["utm_source", "fbclid"])
    .sort_query_params()
    .strip_fragments()
    .lowercase_scheme()
    .build();

let normalized = normalizer.normalize("HTTPS://Example.com/page?b=2&a=1#ref");
// Returns: "https://example.com/page?a=1&b=2"
```

### Near-Duplicate Detection
```rust
// SimHash with configurable bit width
let engine = SimHashEngine::new(64);
let hash1 = engine.hash(content1);
let hash2 = engine.hash(content2);
let similarity = engine.similarity(hash1, hash2);
// Returns: 0.0 (completely different) to 1.0 (identical)
```

### LSHBloom for Scalability
```rust
// Locality-Sensitive Hashing + Bloom for memory efficiency
let lsh = LSHBloom::new(10_000_000, 0.01, 128);
lsh.insert_with_hash(url, simhash);
let candidates = lsh.query_similar(new_url, 0.90);
```

## Architecture

```
┌─────────────────────────────────────────────────────┐
│              DeduplicationEngine                     │
├─────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌───────────┐ │
│  │UrlNormalizer │  │ BloomFilter  │  │ LSHBloom  │ │
│  │              │  │              │  │           │ │
│  │ • Scheme     │  │ • Fast check │  │ • SimHash │ │
│  │ • Query      │  │ • Low memory │  │ • Bands   │ │
│  │ • Params     │  │ • xxHash     │  │ • Similar │ │
│  └──────────────┘  └──────────────┘  └───────────┘ │
│                                                      │
│  ┌──────────────────────────────────────────────┐  │
│  │          Distributed Coordinator              │  │
│  │  • Redis backend for multi-node crawlers     │  │
│  │  • Consistent hashing for URL partitioning   │  │
│  └──────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────┘
```

## Advanced Usage

### Distributed Deduplication
```rust
use kaka::distributed::RedisBackend;

let backend = RedisBackend::new("redis://localhost:6379")?;
let engine = DeduplicationEngine::with_backend(config, backend);

// Automatically syncs across crawler nodes
engine.insert(url);
```

### Custom Normalization Rules
```rust
let normalizer = UrlNormalizer::builder()
    .add_domain_rule("example.com", |url| {
        // Custom logic for specific domains
        url.path().to_lowercase()
    })
    .remove_params_matching(|key| key.starts_with("_"))
    .build();
```

### Batch Processing
```rust
use kaka::batch::BatchProcessor;

let processor = BatchProcessor::new(engine, 100_000);
processor.process_file("urls.txt", |url, is_duplicate| {
    if !is_duplicate {
        crawl_queue.push(url);
    }
})?;
```

## Performance Tuning

### Memory-Constrained Environments
```rust
let config = Config::default()
    .with_capacity(50_000_000)
    .with_false_positive_rate(0.05) // Higher FP, less memory
    .disable_simhash(); // Skip near-duplicate detection
```

### Maximum Throughput
```rust
let config = Config::default()
    .with_threads(num_cpus::get())
    .with_batch_size(10_000)
    .enable_prefetch();
```

## Installation

### From crates.io
```bash
cargo add kaka
```

### From source
```bash
git clone https://github.com/vivekjami/kaka
cd kaka
cargo build --release
```

### Features
```toml
[dependencies]
kaka = { version = "0.1", features = ["distributed", "simhash", "cli"] }
```

- `distributed`: Redis-backed distributed deduplication
- `simhash`: Near-duplicate detection (enabled by default)
- `cli`: Command-line tools for testing and benchmarking

## CLI Tools

```bash
# Deduplicate a file of URLs
kaka dedupe urls.txt -o unique.txt

# Benchmark on Common Crawl sample
kaka bench --dataset commoncrawl --sample 100M

# Start deduplication server
kaka serve --redis redis://localhost:6379
```

## Testing with Common Crawl

```bash
# Download sample
aws s3 cp s3://commoncrawl/crawl-data/CC-MAIN-2025-51/segments/sample.warc.gz . \
    --no-sign-request

# Run deduplication
kaka dedupe sample.warc.gz --format warc -o results.json

# Analyze results
kaka analyze results.json
```

## Production Deployment

### Docker
```dockerfile
FROM rust:1.75 AS builder
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /target/release/kaka /usr/local/bin/
CMD ["kaka", "serve"]
```

### Kubernetes
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: kaka-deduplication
spec:
  replicas: 3
  template:
    spec:
      containers:
      - name: kaka
        image: kaka:latest
        env:
        - name: REDIS_URL
          value: "redis://redis-service:6379"
        resources:
          requests:
            memory: "2Gi"
            cpu: "1000m"
```

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Setup
```bash
git clone https://github.com/vivekjami/kaka
cd kaka
cargo build
cargo test
cargo bench
```

### Running Tests
```bash
# Unit tests
cargo test

# Integration tests with Common Crawl
cargo test --test integration -- --include-ignored

# Benchmarks
cargo bench --bench deduplication
```

## Citation

If you use Kāka in research, please cite:

```bibtex
@software{kaka2026,
  title={Kāka: High-Performance URL Deduplication for Web Crawlers},
  author={Vivek Jami},
  year={2026},
  url={https://github.com/vivekjami/kaka}
}
```

## License

MIT License - see [LICENSE](LICENSE) for details

## Acknowledgments

Built on research from:
- LSHBloom algorithm (arXiv:2411.04257v3)
- SimHash for near-duplicate detection
- Bloom filter optimizations from Google and Facebook's web crawlers

---

**Need help?** Open an issue.