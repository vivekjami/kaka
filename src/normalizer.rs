//! URL normalization logic.
//!
//! This module canonicalizes URLs to ensure semantically equivalent
//! URLs map to the same representation before deduplication.

use std::collections::{HashMap, HashSet};

use url::Url;

/// Domain-specific normalization rule.
type DomainRule = Box<dyn Fn(&Url) -> String>;

/// Configuration flags controlling normalization behavior.
#[derive(Clone, Debug)]
pub struct NormalizerConfig {
    pub lowercase_scheme: bool,
    pub remove_www: bool,
    pub remove_default_port: bool,
    pub sort_query_params: bool,
    pub remove_fragment: bool,
    pub lowercase_hostname: bool,
}

/// URL normalizer.
///
/// Responsible for converting URLs into a canonical form
/// before hashing or deduplication.
pub struct UrlNormalizer {
    tracking_params: HashSet<String>,
    domain_rules: HashMap<String, DomainRule>,
    config: NormalizerConfig,
}

impl UrlNormalizer {
    /// Create a new URL normalizer with default settings.
    pub fn new() -> Self {
        let tracking_params = DEFAULT_TRACKING_PARAMS
            .iter()
            .map(|p| (*p).to_string())
            .collect();

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

    /// Normalize a URL into its canonical representation.
    pub fn normalize(&self, input: &str) -> Result<String, url::ParseError> {
        let url = Url::parse(input)?;

        // Domain-specific overrides
        if let Some(rule) = self.domain_rules.get(url.domain().unwrap_or("")) {
            return Ok(rule(&url));
        }

        let mut out = String::with_capacity(input.len());

        // Scheme
        out.push_str(url.scheme());
        out.push_str("://");

        // Host
        if let Some(host) = url.host_str() {
            let mut host = host.to_string();
            if self.config.lowercase_hostname {
                host.make_ascii_lowercase();
            }
            if self.config.remove_www {
                host = host.strip_prefix("www.").unwrap_or(&host).to_string();
            }
            out.push_str(&host);
        }

        // Port
        if !self.config.remove_default_port
            && let Some(port) = url.port()
        {
            out.push(':');
            out.push_str(&port.to_string());
        }

        // Path
        let path = url.path().trim_end_matches('/');
        if path.is_empty() {
            out.push('/');
        } else {
            out.push_str(path);
        }

        // Query parameters
        if url.query().is_some() {
            let mut params: Vec<(String, String)> = url
                .query_pairs()
                .filter(|(k, _)| !self.tracking_params.contains(k.as_ref()))
                .map(|(k, v)| (k.into_owned(), v.into_owned()))
                .collect();

            if self.config.sort_query_params {
                params.sort_unstable();
            }

            if !params.is_empty() {
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

        // Fragment intentionally dropped if configured
        Ok(out)
    }

    /// Add a tracking query parameter to be removed during normalization.
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

impl Default for UrlNormalizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Default tracking parameters removed during normalization.
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
