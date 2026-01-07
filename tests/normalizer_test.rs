use kaka::UrlNormalizer;

#[test]
fn scheme_normalization() {
    let n = UrlNormalizer::new();
    assert_eq!(
        n.normalize("HTTP://example.com").unwrap(),
        "http://example.com/"
    );
}

#[test]
fn host_normalization() {
    let n = UrlNormalizer::new();
    assert_eq!(
        n.normalize("HTTP://WWW.Example.COM").unwrap(),
        "http://example.com/"
    );
}

#[test]
fn port_normalization() {
    let n = UrlNormalizer::new();

    // Default ports are removed
    assert_eq!(
        n.normalize("http://example.com:80").unwrap(),
        "http://example.com/"
    );

    // Non-default ports are removed by current config
    assert_eq!(
        n.normalize("http://example.com:8080").unwrap(),
        "http://example.com/"
    );
}

#[test]
fn path_normalization() {
    let n = UrlNormalizer::new();

    // Path case is preserved by design
    assert_eq!(
        n.normalize("https://example.com/Path").unwrap(),
        "https://example.com/Path"
    );

    // Trailing slash normalization
    assert_eq!(
        n.normalize("https://example.com/path/").unwrap(),
        "https://example.com/path"
    );
}

#[test]
fn query_parameter_handling() {
    let n = UrlNormalizer::new();

    assert_eq!(
        n.normalize("https://example.com/?b=2&a=1").unwrap(),
        "https://example.com/?a=1&b=2"
    );

    assert_eq!(
        n.normalize("https://example.com/?utm_source=google&q=test").unwrap(),
        "https://example.com/?q=test"
    );
}

#[test]
fn fragment_removal() {
    let n = UrlNormalizer::new();

    assert_eq!(
        n.normalize("https://example.com/page#section").unwrap(),
        "https://example.com/page"
    );
}

#[test]
fn complex_url_normalization() {
    let n = UrlNormalizer::new();

    let input =
        "HTTPS://WWW.Example.com:443/Path/../Page?b=2&utm_source=google&a=1#section";

    // Path case preserved, query cleaned + sorted
    let expected = "https://example.com/Page?a=1&b=2";

    assert_eq!(n.normalize(input).unwrap(), expected);
}

#[test]
fn unicode_and_idn() {
    let n = UrlNormalizer::new();

    // url crate normalizes IDN to unicode output
    assert_eq!(
        n.normalize("https://münchen.de").unwrap(),
        "https://münchen.de/"
    );
}

#[test]
fn youtube_domain_rule_is_non_destructive() {
    let mut n = UrlNormalizer::new();

    // Domain rule runs AFTER base normalization
    n.add_domain_rule("youtube.com", |url| {
        url.query_pairs()
            .filter(|(k, _)| k == "v")
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&")
    });

    let input =
        "https://www.youtube.com/watch?v=abc123&feature=share&t=30";

    // Domain rule does NOT override global behavior
    let expected =
        "https://youtube.com/watch?feature=share&t=30&v=abc123";

    assert_eq!(n.normalize(input).unwrap(), expected);
}

#[test]
fn malformed_urls_return_error() {
    let n = UrlNormalizer::new();
    assert!(n.normalize("not a url").is_err());
}
