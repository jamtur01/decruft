//! Output format and public API tests.
//!
//! Tests JSON/HTML/text output, format consistency, and public API surface.
//! Markdown-specific tests live in markdown.rs.

#![allow(clippy::panic)]

use decruft::{DecruftOptions, parse};

const SAMPLE: &str = r#"<html lang="en">
<head>
    <title>Test Article - Example Blog</title>
    <meta property="og:title" content="Test Article">
    <meta name="author" content="Jane Doe">
    <meta name="description" content="A test article about testing">
    <meta property="og:site_name" content="Example Blog">
    <meta property="article:published_time" content="2025-01-15T10:00:00Z">
</head>
<body>
    <nav><a href="/">Home</a> | <a href="/blog">Blog</a></nav>
    <article>
        <h1>Test Article</h1>
        <p>This is the <strong>first paragraph</strong> with <em>formatted</em> text.</p>
        <p>Here is a <a href="https://example.com">link to example</a> and more content.</p>
        <pre><code class="language-rust">fn main() {
    println!("Hello, world!");
}</code></pre>
        <blockquote><p>A notable quote from someone important.</p></blockquote>
        <ul>
            <li>First item with enough detail to survive extraction filters</li>
            <li>Second item covering another relevant aspect of the topic</li>
            <li>Third item discussing a final point worth considering</li>
        </ul>
        <h2>Second Section</h2>
        <p>More content in the second section with enough words to be meaningful.</p>
    </article>
    <footer>Copyright 2025</footer>
</body>
</html>"#;

fn opts() -> DecruftOptions {
    let mut o = DecruftOptions::default();
    o.url = Some("https://example.com/test-article".into());
    o
}

// ── JSON / metadata ─────────────────────────────────────────────

#[test]
fn metadata_fields() {
    let r = parse(SAMPLE, &opts());
    assert_eq!(r.title, "Test Article");
    assert_eq!(r.author, "Jane Doe");
    assert_eq!(r.site, "Example Blog");
    assert_eq!(r.language, "en");
    assert_eq!(r.domain, "example.com");
    assert!(r.published.contains("2025-01-15"));
    assert!(!r.description.is_empty());
    assert!(r.word_count > 20);
}

#[test]
fn json_serialization() {
    let r = parse(SAMPLE, &opts());
    let json = serde_json::to_string(&r).expect("should serialize");
    assert!(json.contains("\"title\":\"Test Article\""));
    assert!(json.contains("\"content\":"));
}

// ── HTML output ─────────────────────────────────────────────────

#[test]
fn html_semantic_elements() {
    let r = parse(SAMPLE, &opts());
    for tag in [
        "<h1>",
        "<h2>",
        "<strong>",
        "<em>",
        "<blockquote>",
        "<ul>",
        "<li>",
        "<pre>",
        "<code",
    ] {
        assert!(r.content.contains(tag), "missing {tag}");
    }
}

#[test]
fn html_links_preserved() {
    let r = parse(SAMPLE, &opts());
    assert!(r.content.contains("href=\"https://example.com\""));
}

#[test]
fn html_clutter_removed() {
    let r = parse(SAMPLE, &opts());
    assert!(!r.content.contains("Home</a>"));
    assert!(!r.content.contains("Copyright 2025"));
    assert!(!r.content.contains("data-decruft-"));
}

// ── Text output ─────────────────────────────────────────────────

#[test]
fn text_strips_tags() {
    let r = parse(SAMPLE, &opts());
    let text = decruft::strip_html_tags(&r.content);
    assert!(!text.contains("<p>"));
    assert!(!text.contains("<strong>"));
    assert!(text.contains("first paragraph"));
    assert!(text.contains("notable quote"));
}

#[test]
fn strip_html_tags_decodes_entities() {
    let text = decruft::strip_html_tags("<p>AT&amp;T &lt;rocks&gt;</p>");
    assert!(text.contains("AT&T"));
    assert!(text.contains("<rocks>"));
}

// ── Format consistency ──────────────────────────────────────────

#[test]
fn word_count_same_across_formats() {
    let plain = parse(SAMPLE, &opts());
    let mut md_opts = opts();
    md_opts.markdown = true;
    let md = parse(SAMPLE, &md_opts);
    assert_eq!(plain.word_count, md.word_count);
}

#[test]
fn metadata_same_across_formats() {
    let plain = parse(SAMPLE, &opts());
    let mut md_opts = opts();
    md_opts.markdown = true;
    let md = parse(SAMPLE, &md_opts);
    assert_eq!(plain.title, md.title);
    assert_eq!(plain.author, md.author);
    assert_eq!(plain.published, md.published);
}

// ── Public API ──────────────────────────────────────────────────

#[test]
fn parse_with_defaults() {
    let r = decruft::parse_with_defaults(SAMPLE);
    assert!(r.word_count > 20);
    assert!(r.content.contains("first paragraph"));
}

#[test]
fn parse_time_set() {
    let r = parse(SAMPLE, &opts());
    assert!(r.parse_time_ms > 0);
}

#[test]
fn favicon_from_link_icon() {
    let html = r#"<html><head><link rel="icon" href="/favicon.ico"></head>
        <body><article><p>Content.</p></article></body></html>"#;
    let mut o = DecruftOptions::default();
    o.url = Some("https://example.com/page".into());
    assert_eq!(parse(html, &o).favicon, "https://example.com/favicon.ico");
}

#[test]
fn extractor_type_for_github() {
    let html = std::fs::read_to_string(format!(
        "{}/tests/fixtures/defuddle/general--github.com-issue-56.html",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();
    let mut o = DecruftOptions::default();
    o.url = Some("https://github.com/kepano/defuddle/issues/56".into());
    assert_eq!(parse(&html, &o).extractor_type.as_deref(), Some("github"));
}

#[test]
fn include_replies_reduces_content() {
    let html = std::fs::read_to_string(format!(
        "{}/tests/fixtures/defuddle/general--github.com-issue-56.html",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();
    let url = "https://github.com/kepano/defuddle/issues/56";

    let mut with = DecruftOptions::default();
    with.url = Some(url.into());
    with.include_replies = true;

    let mut without = DecruftOptions::default();
    without.url = Some(url.into());
    without.include_replies = false;

    assert!(parse(&html, &without).word_count <= parse(&html, &with).word_count,);
}

#[test]
fn remove_images() {
    let html = r#"<html><body><article>
        <p>Before.</p><img src="photo.jpg" alt="photo"><p>After.</p>
    </article></body></html>"#;
    let mut o = DecruftOptions::default();
    o.remove_images = true;
    let r = parse(html, &o);
    assert!(!r.content.contains("<img"));
    assert!(r.content.contains("Before"));
}
