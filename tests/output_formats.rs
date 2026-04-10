//! Tests for all output formats: JSON, HTML, text, and markdown.
//! Ensures each format produces correct output for the same input.

#![allow(clippy::panic)]

use decruft::{DecruftOptions, parse};

const SAMPLE_HTML: &str = r#"<html lang="en">
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
        <p>This is the <strong>first paragraph</strong> with some <em>formatted</em> text.</p>
        <p>Here is a <a href="https://example.com">link to example</a> and some more content.</p>
        <pre><code class="language-rust">fn main() {
    println!("Hello, world!");
}</code></pre>
        <blockquote><p>A notable quote from someone important.</p></blockquote>
        <p>Here is a list of important considerations for this topic:</p>
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

fn default_opts() -> DecruftOptions {
    let mut o = DecruftOptions::default();
    o.url = Some("https://example.com/test-article".into());
    o
}

// ── JSON output ──────────────────────────────────────────────────

#[test]
fn json_contains_all_metadata_fields() {
    let result = parse(SAMPLE_HTML, &default_opts());

    assert_eq!(result.title, "Test Article");
    assert_eq!(result.author, "Jane Doe");
    assert_eq!(result.site, "Example Blog");
    assert_eq!(result.language, "en");
    assert_eq!(result.domain, "example.com");
    assert!(result.published.contains("2025-01-15"));
    assert!(!result.description.is_empty());
    assert!(result.word_count > 20);
}

#[test]
fn json_content_is_html() {
    let result = parse(SAMPLE_HTML, &default_opts());

    assert!(result.content.contains("<p>"), "content should be HTML");
    assert!(result.content.contains("<strong>"));
    assert!(result.content.contains("<a href="));
}

#[test]
fn json_excludes_navigation_and_footer() {
    let result = parse(SAMPLE_HTML, &default_opts());

    assert!(!result.content.contains("Home</a>"));
    assert!(!result.content.contains("Copyright 2025"));
}

#[test]
fn json_serializes_correctly() {
    let result = parse(SAMPLE_HTML, &default_opts());
    let json = serde_json::to_string(&result).expect("should serialize");

    assert!(json.contains("\"title\":\"Test Article\""));
    assert!(json.contains("\"word_count\":"));
    assert!(json.contains("\"content\":"));
}

// ── HTML output ──────────────────────────────────────────────────

#[test]
fn html_preserves_semantic_elements() {
    let result = parse(SAMPLE_HTML, &default_opts());

    assert!(result.content.contains("<h1>"));
    assert!(result.content.contains("<h2>"));
    assert!(result.content.contains("<strong>"));
    assert!(result.content.contains("<em>"));
    assert!(result.content.contains("<blockquote>"));
    assert!(result.content.contains("<ul>"));
    assert!(result.content.contains("<li>"));
}

#[test]
fn html_preserves_code_blocks() {
    let result = parse(SAMPLE_HTML, &default_opts());

    assert!(result.content.contains("<pre>"));
    assert!(result.content.contains("<code"));
    assert!(result.content.contains("println!"));
}

#[test]
fn html_preserves_links() {
    let result = parse(SAMPLE_HTML, &default_opts());

    assert!(result.content.contains("href=\"https://example.com\""));
    assert!(result.content.contains("link to example"));
}

#[test]
fn html_strips_non_allowed_attributes() {
    let result = parse(SAMPLE_HTML, &default_opts());

    // Navigation links should be gone
    assert!(!result.content.contains("Home</a>"));
    // No internal decruft attributes
    assert!(!result.content.contains("data-decruft-"));
}

// ── Text output ──────────────────────────────────────────────────

#[test]
fn text_strips_all_html_tags() {
    let result = parse(SAMPLE_HTML, &default_opts());
    let text = decruft::dom::strip_html_tags(&result.content);

    assert!(!text.contains("<p>"), "text should not contain <p>");
    assert!(
        !text.contains("<strong>"),
        "text should not contain <strong>"
    );
    assert!(!text.contains("<a "), "text should not contain <a");
    assert!(!text.contains("</"), "text should not contain closing tags");
}

#[test]
fn text_preserves_actual_content() {
    let result = parse(SAMPLE_HTML, &default_opts());
    let text = decruft::dom::strip_html_tags(&result.content);

    assert!(text.contains("first paragraph"));
    assert!(text.contains("formatted"));
    assert!(text.contains("notable quote"));
    assert!(text.contains("Second Section"));
}

// ── Markdown output ──────────────────────────────────────────────

#[test]
fn markdown_option_converts_content() {
    let mut opts = default_opts();
    opts.markdown = true;
    let result = parse(SAMPLE_HTML, &opts);

    // Content should be markdown, not HTML
    assert!(
        !result.content.contains("<p>"),
        "markdown content should not have <p> tags: {}",
        &result.content[..200.min(result.content.len())]
    );
    assert!(
        result.content.contains("**first paragraph**") || result.content.contains("*formatted*"),
        "markdown should have formatting"
    );
}

#[test]
fn markdown_preserves_code_fences() {
    let mut opts = default_opts();
    opts.markdown = true;
    let result = parse(SAMPLE_HTML, &opts);

    assert!(
        result.content.contains("```") || result.content.contains("    fn main"),
        "markdown should have code blocks"
    );
    assert!(result.content.contains("println!"));
}

#[test]
fn markdown_converts_links() {
    let mut opts = default_opts();
    opts.markdown = true;
    let result = parse(SAMPLE_HTML, &opts);

    assert!(
        result
            .content
            .contains("[link to example](https://example.com)")
            || result.content.contains("[link to example]"),
        "markdown should have link syntax"
    );
}

#[test]
fn markdown_converts_lists() {
    let mut opts = default_opts();
    opts.markdown = true;
    let result = parse(SAMPLE_HTML, &opts);

    assert!(
        result.content.contains("- First item")
            || result.content.contains("* First item")
            || result.content.contains("*   First item")
            || result.content.contains("1. First item"),
        "markdown should have list syntax, got: {}",
        &result.content[..500.min(result.content.len())]
    );
}

#[test]
fn markdown_converts_headings() {
    let mut opts = default_opts();
    opts.markdown = true;
    let result = parse(SAMPLE_HTML, &opts);

    assert!(
        result.content.contains("# ") || result.content.contains("## "),
        "markdown should have heading syntax"
    );
}

#[test]
fn markdown_converts_blockquotes() {
    let mut opts = default_opts();
    opts.markdown = true;
    let result = parse(SAMPLE_HTML, &opts);

    assert!(
        result.content.contains("> "),
        "markdown should have blockquote syntax"
    );
}

#[test]
fn separate_markdown_includes_both() {
    let mut opts = default_opts();
    opts.separate_markdown = true;
    let result = parse(SAMPLE_HTML, &opts);

    // content should still be HTML
    assert!(result.content.contains("<p>"), "content should remain HTML");

    // content_markdown should be markdown
    let md = result
        .content_markdown
        .as_ref()
        .expect("should have markdown");
    assert!(!md.contains("<p>"), "markdown should not have HTML tags");
    assert!(!md.is_empty());
}

#[test]
fn no_markdown_option_returns_none() {
    let result = parse(SAMPLE_HTML, &default_opts());
    assert!(
        result.content_markdown.is_none(),
        "content_markdown should be None without markdown option"
    );
}

// ── Format consistency ───────────────────────────────────────────

#[test]
fn all_formats_have_same_word_count() {
    let result_plain = parse(SAMPLE_HTML, &default_opts());

    let mut md_opts = default_opts();
    md_opts.markdown = true;
    let result_md = parse(SAMPLE_HTML, &md_opts);

    let mut sep_opts = default_opts();
    sep_opts.separate_markdown = true;
    let result_sep = parse(SAMPLE_HTML, &sep_opts);

    // Word counts should be identical regardless of output format
    assert_eq!(result_plain.word_count, result_md.word_count);
    assert_eq!(result_plain.word_count, result_sep.word_count);
}

#[test]
fn all_formats_have_same_metadata() {
    let result_plain = parse(SAMPLE_HTML, &default_opts());

    let mut md_opts = default_opts();
    md_opts.markdown = true;
    let result_md = parse(SAMPLE_HTML, &md_opts);

    assert_eq!(result_plain.title, result_md.title);
    assert_eq!(result_plain.author, result_md.author);
    assert_eq!(result_plain.site, result_md.site);
    assert_eq!(result_plain.published, result_md.published);
}

// ── Real-world format tests ──────────────────────────────────────

#[test]
fn wikipedia_all_formats() {
    let Ok(html) = std::fs::read_to_string(format!(
        "{}/tests/fixtures/wikipedia_bengaluru.html",
        env!("CARGO_MANIFEST_DIR")
    )) else {
        return;
    };

    let mut opts = DecruftOptions::default();
    opts.url = Some("https://en.wikipedia.org/wiki/Bengaluru".into());

    // JSON
    let result = parse(&html, &opts);
    assert!(result.word_count > 5000);
    assert!(result.content.contains("<p>"));

    // HTML is valid
    assert!(result.content.contains("Bengaluru"));
    assert!(!result.content.contains("data-decruft-"));

    // Markdown
    opts.markdown = true;
    let md_result = parse(&html, &opts);
    assert!(
        !md_result.content.contains("<p>"),
        "markdown should strip tags"
    );
    assert!(md_result.content.contains("Bengaluru"));
    assert!(md_result.content.contains("**"), "should have bold markers");
}

#[test]
fn complex_blog_all_formats() {
    let Ok(html) = std::fs::read_to_string(format!(
        "{}/tests/fixtures/complex_blog.html",
        env!("CARGO_MANIFEST_DIR")
    )) else {
        return;
    };

    let mut opts = DecruftOptions::default();
    opts.url = Some("https://example.com/article".into());

    let result = parse(&html, &opts);
    assert!(result.content.contains("Rust's ownership system"));

    // Markdown
    opts.markdown = true;
    let md_result = parse(&html, &opts);
    assert!(md_result.content.contains("Rust's ownership system"));
    assert!(
        md_result.content.contains("```"),
        "markdown should have code fences"
    );
}
