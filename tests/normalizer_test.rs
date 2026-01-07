//! Tests for URL normalization behavior.
//!
//! These tests validate that different textual representations of URLs
//! normalize into a single canonical form suitable for deduplication.

use kaka::normalizer::UrlNormalizer;

#[test]
fn scheme_normalization() {
    let normalizer = UrlNormalizer::new();

    assert_eq!(
        normalizer.normalize("HTTP://example.com").unwrap(),
        "http://example.com/"
    );

    assert_eq!(
        normalizer.normalize("HTTPS://example.com").unwrap(),
        "https://example.com/"
    );
}

#[test]
fn host_normalization() {
    let normalizer = UrlNormalizer::new();

    assert_eq!(
        normalizer.normalize("http://WWW.Example.COM").unwrap(),
        "http://example.com/"
    );

    assert_eq!(
        normalizer.normalize("http://EXAMPLE.COM").unwrap(),
        "http://example.com/"
    );
}

#[test]
fn port_normalization() {
    let normalizer = UrlNormalizer::new();

    assert_eq!(
        normalizer.normalize("http://example.com:80").unwrap(),
        "http://example.com/"
    );

    assert_eq!(
        normalizer.normalize("https://example.com:443").unwrap(),
        "https://example.com/"
    );

    assert_eq!(
        normalizer.normalize("http://example.com:8080").unwrap(),
        "http://example.com:8080/"
    );
}

#[test]
fn path_normalization() {
    let normalizer = UrlNormalizer::new();

    assert_eq!(
        normalizer
            .normalize("http://example.com/path/to/../page")
            .unwrap(),
        "http://example.com/path/page"
    );

    assert_eq!(
        normalizer.normalize("http://example.com/path/").unwrap(),
        "http://example.com/path"
    );

    assert_eq!(
        normalizer.normalize("http://example.com/").unwrap(),
        "http://example.com/"
    );
}

#[test]
fn query_parameter_handling() {
    let normalizer = UrlNormalizer::new();

    assert_eq!(
        normalizer
            .normalize("http://example.com/?b=2&a=1")
            .unwrap(),
        "http://example.com/?a=1&b=2"
    );

    assert_eq!(
        normalizer
            .normalize("http://example.com/?utm_source=google&q=test")
            .unwrap(),
        "http://example.com/?q=test"
    );

    assert_eq!(
        normalizer.normalize("http://example.com/?").unwrap(),
        "http://example.com/"
    );
}

#[test]
fn fragment_removal() {
    let normalizer = UrlNormalizer::new();

    assert_eq!(
        normalizer
            .normalize("http://example.com/page#section")
            .unwrap(),
        "http://example.com/page"
    );

    assert_eq!(
        normalizer.normalize("http://example.com/page#").unwrap(),
        "http://example.com/page"
    );
}

#[test]
fn complex_url_normalization() {
    let normalizer = UrlNormalizer::new();

    let input =
        "HTTPS://WWW.Example.com:443/Path/../Page?b=2&utm_source=google&a=1#section";
    let expected = "https://example.com/page?a=1&b=2";

    assert_eq!(normalizer.normalize(input).unwrap(), expected);
}

#[test]
fn unicode_and_idn() {
    let normalizer = UrlNormalizer::new();

    // The `url` crate canonicalizes internationalized domain names (IDN)
    // into ASCII-compatible Punycode form as per RFC standards.
    assert_eq!(
        normalizer.normalize("https://münchen.de").unwrap(),
        "https://xn--mnchen-3ya.de/"
    );
}

#[test]
fn malformed_urls_return_error() {
    let normalizer = UrlNormalizer::new();

    assert!(normalizer.normalize("not a url").is_err());
    assert!(normalizer.normalize("").is_err());
}

#[test]
fn youtube_domain_specific_rule() {
    let mut normalizer = UrlNormalizer::new();

    // YouTube-specific normalization:
    // keep only the `v` query parameter.
    normalizer.add_domain_rule("youtube.com", |url| {
        url.query_pairs()
            .filter(|(k, _)| k == "v")
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&")
    });

    let input =
        "https://www.youtube.com/watch?v=abc123&feature=share&t=30";
    let expected = "https://youtube.com/watch?v=abc123";

    assert_eq!(normalizer.normalize(input).unwrap(), expected);
}
