//! URL normalization module.
//!
//! This module canonicalizes URLs so that semantically identical URLs
//! map to the same string representation. Performance is critical here,
//! as normalization typically dominates crawler deduplication pipelines.

use std::collections::{HashMap, HashSet};

use url::Url;

/// URL normalizer with configurable rules.
pub struct UrlNormalizer {
    /// Query parameters that should be removed (tracking params).
    tracking_params: HashSet<String>,

    /// Optional domain-specific normalization rules.
    domain_rules: HashMap<String, Box<dyn Fn(&Url) -> String>>,

    /// Normalization configuration flags.
    config: NormalizerConfig,
}

/// Normalization configuration.
/// Kept simple to allow compiler optimizations.
#[derive(Clone, Copy)]
pub struct NormalizerConfig {
    pub lowercase_scheme: bool,
    pub remove_www: bool,
    pub remove_default_port: bool,
    pub sort_query_params: bool,
    pub remove_fragment: bool,
    pub lowercase_hostname: bool,
}

impl UrlNormalizer {
    /// Create a new URL normalizer with default settings.
    pub fn new() -> Self {
        let mut tracking_params = HashSet::with_capacity(DEFAULT_TRACKING_PARAMS.len());
        for p in DEFAULT_TRACKING_PARAMS {
            tracking_params.insert((*p).to_string());
        }

        Self {
            tracking_params,
            domain_rules: HashMap::new(),
            config: NormalizerConfig {
                lowercase_scheme: true,
                remove_www: true,
                remove_default_port: true,
                sort_query_params: true,
                remove_fragment: true,
                lowercase_hostname: true,
            },
        }
    }

    /// Normalize a URL into its canonical form.
    ///
    /// This function is performance-sensitive and structured to:
    /// - Avoid unnecessary allocations
    /// - Skip expensive work for simple URLs
    /// - Minimize query sorting overhead
    pub fn normalize(&self, input: &str) -> Result<String, url::ParseError> {
        let url = Url::parse(input)?;

        // ---- Domain-specific override (fast exit) ----
        if let Some(rule) = self.domain_rules.get(url.domain().unwrap_or("")) {
            return Ok(rule(&url));
        }

        // ---- Build output incrementally into a single buffer ----
        let mut out = String::with_capacity(input.len());

        // ---- Scheme ----
        if self.config.lowercase_scheme {
            out.push_str(url.scheme());
        } else {
            out.push_str(url.scheme());
        }
        out.push_str("://");

        // ---- Host ----
        if let Some(host) = url.host_str() {
            let host = if self.config.lowercase_hostname {
                host.to_ascii_lowercase()
            } else {
                host.to_string()
            };

            if self.config.remove_www && host.starts_with("www.") {
                out.push_str(&host[4..]);
            } else {
                out.push_str(&host);
            }
        }

        // ---- Port ----
        if let Some(port) = url.port() {
            let is_default = matches!(
                (url.scheme(), port),
                ("http", 80) | ("https", 443)
            );

            if !(self.config.remove_default_port && is_default) {
                out.push(':');
                out.push_str(&port.to_string());
            }
        }

        // ---- Path ----
        let path = url.path();
        if path != "/" {
            // Avoid allocations for trivial paths
            out.push_str(path.trim_end_matches('/'));
        } else {
            out.push('/');
        }

         // ---- Query ----
        if let Some(_) = url.query() {
            // Store owned strings to satisfy Rust lifetimes
            let mut params: Vec<(String, String)> = Vec::with_capacity(4);

            for (k, v) in url.query_pairs() {
                if !self.tracking_params.contains(k.as_ref()) {
                    params.push((k.into_owned(), v.into_owned()));
                }
            }

            // Fast path: 0 or 1 parameter → no sort needed
            if !params.is_empty() {
                if self.config.sort_query_params && params.len() > 1 {
                    params.sort_unstable_by(|a, b| a.0.cmp(&b.0));
                }

                out.push('?');
                for (i, (k, v)) in params.iter().enumerate() {
                    if i > 0 {
                        out.push('&');
                    }
                    out.push_str(k);
                    out.push('=');
                    out.push_str(v);
                }
            }
        }

        // ---- Fragment ----
        // Fragment intentionally dropped if configured

        Ok(out)
    }

    /// Add a tracking parameter to be removed during normalization.
    pub fn add_tracking_param(&mut self, param: &str) {
        self.tracking_params.insert(param.to_string());
    }

    /// Add a domain-specific normalization rule.
    pub fn add_domain_rule<F>(&mut self, domain: &str, rule: F)
    where
        F: Fn(&Url) -> String + 'static,
    {
        self.domain_rules.insert(domain.to_string(), Box::new(rule));
    }
}

/// Default tracking parameters removed from URLs.
const DEFAULT_TRACKING_PARAMS: &[&str] = &[
    "utm_source",
    "utm_medium",
    "utm_campaign",
    "utm_content",
    "utm_term",
    "fbclid",
    "gclid",
    "msclkid",
    "_ga",
    "_gl",
    "mc_cid",
    "mc_eid",
    "ref",
    "referrer",
];
