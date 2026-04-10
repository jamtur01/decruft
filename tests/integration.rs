#![allow(clippy::panic)]

use decruft::{DecruftOptions, parse};

fn opts() -> DecruftOptions {
    let mut o = DecruftOptions::default();
    o.url = Some("https://example.com/article".into());
    o
}

fn opts_debug() -> DecruftOptions {
    let mut o = DecruftOptions::default();
    o.url = Some("https://example.com/article".into());
    o.debug = true;
    o
}

fn load_fixture(name: &str) -> String {
    let path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("fixture {name} not found at {path}: {e}"))
}

// ── Complex blog post tests ──────────────────────────────────────

#[test]
fn blog_extracts_article_content() {
    let html = load_fixture("complex_blog.html");
    let result = parse(&html, &opts());

    assert!(
        result.content.contains("Rust's ownership system"),
        "should contain the article's opening paragraph"
    );
    assert!(
        result.content.contains("Borrowing and References"),
        "should contain the h2 heading"
    );
    assert!(
        result.content.contains("Lifetimes"),
        "should contain the lifetimes section"
    );
}

#[test]
fn blog_preserves_code_blocks() {
    let html = load_fixture("complex_blog.html");
    let result = parse(&html, &opts());

    assert!(
        result.content.contains("<pre><code"),
        "should preserve code blocks"
    );
    assert!(
        result.content.contains("String::from"),
        "should keep code content"
    );
}

#[test]
fn blog_removes_navigation() {
    let html = load_fixture("complex_blog.html");
    let result = parse(&html, &opts());

    assert!(
        !result.content.contains("main-nav"),
        "should remove navigation"
    );
    assert!(
        !result.content.contains("Home</a>"),
        "should not contain nav links from header"
    );
}

#[test]
fn blog_removes_ads() {
    let html = load_fixture("complex_blog.html");
    let result = parse(&html, &opts());

    assert!(
        !result.content.contains("ad-banner"),
        "should remove ad banners"
    );
    assert!(
        !result.content.contains("sidebar-ad"),
        "should remove sidebar ads"
    );
    assert!(
        !result.content.contains("banner-728x90"),
        "should remove ad images"
    );
}

#[test]
fn blog_removes_sidebar() {
    let html = load_fixture("complex_blog.html");
    let result = parse(&html, &opts());

    assert!(
        !result.content.contains("Popular Posts"),
        "should remove sidebar popular posts widget"
    );
    assert!(
        !result.content.contains("Follow Us"),
        "should remove sidebar social links"
    );
}

#[test]
fn blog_removes_cookie_consent() {
    let html = load_fixture("complex_blog.html");
    let result = parse(&html, &opts());

    assert!(
        !result.content.contains("cookie"),
        "should remove cookie consent banner"
    );
}

#[test]
fn blog_removes_footer() {
    let html = load_fixture("complex_blog.html");
    let result = parse(&html, &opts());

    assert!(
        !result.content.contains("Privacy Policy"),
        "should remove footer links"
    );
    assert!(
        !result.content.contains("All rights reserved"),
        "should remove copyright notice"
    );
}

#[test]
fn blog_removes_share_buttons() {
    let html = load_fixture("complex_blog.html");
    let result = parse(&html, &opts());

    assert!(
        !result.content.contains("share-twitter"),
        "should remove share buttons"
    );
    assert!(
        !result.content.contains("Share this article"),
        "should remove share heading"
    );
}

#[test]
fn blog_removes_comments() {
    let html = load_fixture("complex_blog.html");
    let result = parse(&html, &opts());

    assert!(
        !result.content.contains("Comments (12)"),
        "should remove comments section"
    );
}

#[test]
fn blog_removes_newsletter_cta() {
    let html = load_fixture("complex_blog.html");
    let result = parse(&html, &opts());

    assert!(
        !result.content.contains("newsletter"),
        "should remove newsletter signup"
    );
}

#[test]
fn blog_removes_related_posts() {
    let html = load_fixture("complex_blog.html");
    let result = parse(&html, &opts());

    assert!(
        !result.content.contains("You Might Also Like"),
        "should remove related posts heading"
    );
}

#[test]
fn blog_removes_hidden_elements() {
    let html = load_fixture("complex_blog.html");
    let result = parse(&html, &opts());

    assert!(
        !result.content.contains("promo-popup"),
        "should remove hidden modal"
    );
    assert!(
        !result.content.contains("pixel.gif"),
        "should remove tracking pixels"
    );
}

#[test]
fn blog_extracts_metadata() {
    let html = load_fixture("complex_blog.html");
    let result = parse(&html, &opts());

    assert_eq!(result.title, "Understanding Rust Ownership");
    assert_eq!(result.author, "Alice Chen");
    assert!(
        result.description.contains("ownership model"),
        "description: {}",
        result.description
    );
    assert_eq!(result.site, "TechBlog Daily");
    assert_eq!(
        result.image,
        "https://techblog.example.com/images/rust-ownership.jpg"
    );
}

#[test]
fn blog_extracts_published_date() {
    let html = load_fixture("complex_blog.html");
    let result = parse(&html, &opts());

    assert!(
        result.published.contains("2025-03-15"),
        "published: {}",
        result.published
    );
}

#[test]
fn blog_extracts_schema_org() {
    let html = load_fixture("complex_blog.html");
    let result = parse(&html, &opts());

    assert!(
        result.schema_org_data.is_some(),
        "should have schema.org data"
    );
    let schema = result.schema_org_data.as_ref().expect("schema");
    assert_eq!(
        schema.get("@type").and_then(|v| v.as_str()),
        Some("Article")
    );
}

#[test]
fn blog_has_reasonable_word_count() {
    let html = load_fixture("complex_blog.html");
    let result = parse(&html, &opts());

    assert!(
        result.word_count > 200,
        "word count should be > 200, got {}",
        result.word_count
    );
    assert!(
        result.word_count < 1000,
        "word count should be < 1000 (no clutter), got {}",
        result.word_count
    );
}

// ── News article tests ───────────────────────────────────────────

#[test]
fn news_extracts_article_body() {
    let html = load_fixture("news_article.html");
    let result = parse(&html, &opts());

    assert!(
        result.content.contains("Marine biologists"),
        "should contain article lead"
    );
    assert!(
        result.content.contains("Aurelia profundis"),
        "should contain species name"
    );
    assert!(
        result.content.contains("The Three New Species"),
        "should contain subheading"
    );
    assert!(
        result.content.contains("chemosynthetic"),
        "should contain science content"
    );
}

#[test]
fn news_preserves_blockquotes() {
    let html = load_fixture("news_article.html");
    let result = parse(&html, &opts());

    assert!(
        result.content.contains("<blockquote"),
        "should preserve blockquotes"
    );
    assert!(
        result.content.contains("last great frontier"),
        "should keep quote content"
    );
}

#[test]
fn news_preserves_figures() {
    let html = load_fixture("news_article.html");
    let result = parse(&html, &opts());

    assert!(
        result.content.contains("<figure"),
        "should preserve figure elements"
    );
    assert!(
        result.content.contains("figcaption"),
        "should preserve figcaptions"
    );
}

#[test]
fn news_removes_inline_ads() {
    let html = load_fixture("news_article.html");
    let result = parse(&html, &opts());

    assert!(
        !result.content.contains("inline-ad"),
        "should remove inline ads"
    );
    assert!(
        !result.content.contains("ADVERTISEMENT"),
        "should remove ad labels"
    );
}

#[test]
fn news_removes_sidebars() {
    let html = load_fixture("news_article.html");
    let result = parse(&html, &opts());

    assert!(
        !result.content.contains("Trending Now"),
        "should remove trending sidebar"
    );
    assert!(
        !result.content.contains("Recommended For You"),
        "should remove recommendations sidebar"
    );
    assert!(
        !result.content.contains("Sponsored Stories"),
        "should remove sponsored content"
    );
}

#[test]
fn news_removes_breaking_ticker() {
    let html = load_fixture("news_article.html");
    let result = parse(&html, &opts());

    assert!(
        !result.content.contains("BREAKING"),
        "should remove breaking news ticker"
    );
    assert!(
        !result.content.contains("Stock markets"),
        "should remove ticker items"
    );
}

#[test]
fn news_removes_more_stories() {
    let html = load_fixture("news_article.html");
    let result = parse(&html, &opts());

    assert!(
        !result.content.contains("More Science Stories"),
        "should remove more stories section"
    );
}

#[test]
fn news_removes_notification_popup() {
    let html = load_fixture("news_article.html");
    let result = parse(&html, &opts());

    assert!(
        !result.content.contains("Enable notifications"),
        "should remove notification popup"
    );
}

#[test]
fn news_extracts_metadata() {
    let html = load_fixture("news_article.html");
    let result = parse(&html, &opts());

    assert_eq!(
        result.title,
        "Scientists Discover New Species in Deep Ocean"
    );
    assert_eq!(result.author, "Sarah Mitchell");
    assert_eq!(result.site, "World News Today");
    assert!(result.published.contains("2025-06-20"));
}

#[test]
fn news_extracts_graph_schema() {
    let html = load_fixture("news_article.html");
    let result = parse(&html, &opts());

    assert!(result.schema_org_data.is_some());
    let schema = result.schema_org_data.as_ref().expect("schema");
    // Should have flattened the @graph into an array
    assert!(schema.is_array(), "schema should be a flattened array");
}

// ── Debug mode tests ─────────────────────────────────────────────

#[test]
fn debug_mode_includes_removals() {
    let html = load_fixture("complex_blog.html");
    let result = parse(&html, &opts_debug());

    assert!(result.debug.is_some(), "debug info should be present");
    let debug = result.debug.as_ref().expect("debug");
    assert!(!debug.removals.is_empty(), "should have recorded removals");
    assert!(
        !debug.content_selector.is_empty(),
        "should have content selector path"
    );
}

#[test]
fn debug_mode_includes_meta_tags() {
    let html = load_fixture("complex_blog.html");
    let result = parse(&html, &opts_debug());

    assert!(
        result.meta_tags.is_some(),
        "debug mode should include meta tags"
    );
    let tags = result.meta_tags.as_ref().expect("tags");
    assert!(!tags.is_empty(), "should have meta tags");
}

// ── Option toggle tests ──────────────────────────────────────────

#[test]
fn no_images_removes_all_images() {
    let html = load_fixture("news_article.html");
    let mut opts = DecruftOptions::default();
    opts.url = Some("https://example.com".into());
    opts.remove_images = true;
    let result = parse(&html, &opts);

    assert!(!result.content.contains("<img"), "should remove all images");
}

#[test]
fn custom_selector_overrides_detection() {
    let html = load_fixture("complex_blog.html");
    let mut opts = DecruftOptions::default();
    opts.url = Some("https://example.com".into());
    opts.content_selector = Some("article.post-content".into());
    let result = parse(&html, &opts);

    assert!(
        result.content.contains("Rust's ownership system"),
        "custom selector should still extract article"
    );
}

#[test]
fn disabling_all_filters_preserves_more_content() {
    let html = load_fixture("complex_blog.html");
    let strict = parse(&html, &opts());
    let relaxed = parse(&html, &{
        let mut o = DecruftOptions::default();
        o.url = Some("https://example.com".into());
        o.remove_exact_selectors = false;
        o.remove_partial_selectors = false;
        o.remove_hidden_elements = false;
        o.remove_low_scoring = false;
        o.remove_content_patterns = false;
        o
    });

    assert!(
        relaxed.word_count >= strict.word_count,
        "relaxed ({}) should have >= words than strict ({})",
        relaxed.word_count,
        strict.word_count
    );
}

// ── Edge case tests ──────────────────────────────────────────────

#[test]
fn empty_document() {
    let result = parse("", &DecruftOptions::default());
    assert!(result.content.is_empty() || result.word_count == 0);
}

#[test]
fn minimal_document() {
    let html = "<html><body><p>Hello world</p></body></html>";
    let result = parse(html, &DecruftOptions::default());
    assert!(result.content.contains("Hello world"));
}

#[test]
fn document_with_only_navigation() {
    let html = r#"<html><body>
        <nav><ul><li><a href="/">Home</a></li><li><a href="/about">About</a></li></ul></nav>
        <footer><p>Copyright 2025</p></footer>
    </body></html>"#;
    let result = parse(html, &DecruftOptions::default());
    // Should still return something, even if it's mostly empty
    assert!(result.word_count < 10);
}

#[test]
fn cjk_word_counting() {
    let html = "<html><body><article><p>This article discusses \u{4E2D}\u{6587}\u{5185}\u{5BB9} which is Chinese content with English mixed in.</p></article></body></html>";
    let result = parse(html, &DecruftOptions::default());
    assert!(
        result.word_count > 8,
        "should count CJK characters as words, got {}",
        result.word_count
    );
}

#[test]
fn preserves_semantic_html() {
    let html = r"<html><body>
        <article>
            <h1>Title</h1>
            <p>First paragraph with <strong>bold</strong> and <em>italic</em> text.</p>
            <ul><li>Item one</li><li>Item two</li></ul>
            <table><tr><th>Header</th></tr><tr><td>Data</td></tr></table>
        </article>
    </body></html>";
    let result = parse(html, &DecruftOptions::default());

    assert!(result.content.contains("<strong>bold</strong>"));
    assert!(result.content.contains("<em>italic</em>"));
    assert!(result.content.contains("<ul>"));
    assert!(result.content.contains("<table>"));
}

#[test]
fn handles_multiple_articles() {
    let html = r"<html><body>
        <article>
            <h1>Main Article</h1>
            <p>This is the main article with substantial content that should be the primary extraction target because it has many more words than the sidebar teaser.</p>
            <p>Additional paragraph with more meaningful content about the topic being discussed in this important article.</p>
        </article>
        <aside>
            <article>
                <h2>Sidebar Teaser</h2>
                <p>Short teaser</p>
            </article>
        </aside>
    </body></html>";
    let result = parse(html, &DecruftOptions::default());

    assert!(
        result.content.contains("Main Article"),
        "should prefer the main article"
    );
}

// ── Real-world fixture tests (if available) ──────────────────────

#[test]
fn paulgraham_if_available() {
    let path = format!(
        "{}/tests/fixtures/paulgraham.html",
        env!("CARGO_MANIFEST_DIR")
    );
    let html =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("fixture missing: {path}: {e}"));

    let result = parse(&html, &{
        let mut o = DecruftOptions::default();
        o.url = Some("https://www.paulgraham.com/read.html".into());
        o
    });

    assert!(
        result.word_count > 100,
        "PG essays should have substantial content, got {}",
        result.word_count
    );
    assert!(
        result.title.len() > 2,
        "should extract a title: '{}'",
        result.title
    );
}

#[test]
fn rust_blog_if_available() {
    let path = format!(
        "{}/tests/fixtures/rust_blog.html",
        env!("CARGO_MANIFEST_DIR")
    );
    let html =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("fixture missing: {path}: {e}"));

    let result = parse(&html, &{
        let mut o = DecruftOptions::default();
        o.url = Some("https://blog.rust-lang.org/2024/11/28/Rust-2024-Edition.html".into());
        o
    });

    assert!(
        result.word_count > 100,
        "Rust blog should have content, got {}",
        result.word_count
    );
    assert!(!result.content.contains("footer"));
}

#[test]
fn wikipedia_if_available() {
    let path = format!(
        "{}/tests/fixtures/wikipedia.html",
        env!("CARGO_MANIFEST_DIR")
    );
    let html =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("fixture missing: {path}: {e}"));

    let result = parse(&html, &{
        let mut o = DecruftOptions::default();
        o.url = Some("https://en.wikipedia.org/wiki/Rust_(programming_language)".into());
        o
    });

    assert!(
        result.word_count > 500,
        "Wikipedia should have lots of content, got {}",
        result.word_count
    );
    assert!(
        result.language == "en",
        "should detect English, got '{}'",
        result.language
    );
}

#[test]
fn wikipedia_bengaluru_extraction() {
    let path = format!(
        "{}/tests/fixtures/wikipedia_bengaluru.html",
        env!("CARGO_MANIFEST_DIR")
    );
    let html =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("fixture missing: {path}: {e}"));

    let result = parse(&html, &{
        let mut o = DecruftOptions::default();
        o.url = Some("https://en.wikipedia.org/wiki/Bengaluru".into());
        o
    });

    // Title should strip "- Wikipedia" suffix (domain-derived site name)
    assert_eq!(result.title, "Bengaluru", "title: {}", result.title);

    // Metadata
    assert_eq!(result.language, "en");
    assert_eq!(result.domain, "en.wikipedia.org");

    // Content should include the article text
    assert!(
        result.content.contains("Bengaluru"),
        "should contain city name"
    );
    assert!(
        result.content.contains("Karnataka"),
        "should contain state name"
    );

    // Should NOT contain navboxes, infoboxes, reference lists
    assert!(
        !result.content.contains("class=\"navbox"),
        "should strip navbox tables"
    );
    assert!(
        !result.content.contains("class=\"infobox"),
        "should strip infobox tables"
    );
    assert!(
        !result.content.contains("class=\"reflist"),
        "should strip reference lists"
    );

    // Should NOT leak internal attributes
    assert!(
        !result.content.contains("data-decruft-"),
        "should strip internal data-decruft attributes"
    );

    // Word count should be reasonable (within range of defuddle's ~13400)
    assert!(
        result.word_count > 8000,
        "too few words: {}",
        result.word_count
    );
    assert!(
        result.word_count < 25000,
        "too many words (including clutter?): {}",
        result.word_count
    );
}
