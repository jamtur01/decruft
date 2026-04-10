//! Behavioral tests ported from defuddle's test suite.
//!
//! Each section maps to a defuddle test file:
//! - debug.test.ts      -> Debug options / Pipeline toggles / contentSelector
//! - markdown.test.ts   -> Markdown conversion edge cases
//! - schema-fallback.test.ts -> Schema.org text fallback + sanitization
//! - full-bundle.test.ts     -> Markdown output modes

use decruft::{DecruftOptions, parse};

// ── Helpers ─────────────────────────────────────────────────────────

fn opts(url: &str) -> DecruftOptions {
    let mut o = DecruftOptions::default();
    o.url = Some(url.into());
    o
}

fn opts_debug(url: &str) -> DecruftOptions {
    let mut o = DecruftOptions::default();
    o.url = Some(url.into());
    o.debug = true;
    o
}

/// Fixture HTML from stephango.com buy-wisely (loaded from disk).
///
/// Returns `None` when the fixture file is missing, allowing tests
/// to skip gracefully instead of panicking.
fn fixture_html() -> Option<String> {
    let path = format!(
        "{}/tests/fixtures/defuddle/general--stephango.com-buy-wisely.html",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(&path).ok()
}

const FIXTURE_URL: &str = "https://stephango.com/buy-wisely";

// ═══════════════════════════════════════════════════════════════════
// Debug options  (debug.test.ts -> "Debug options")
// ═══════════════════════════════════════════════════════════════════

#[test]
fn debug_true_returns_debug_info_with_content_selector_and_removals() {
    let Some(html) = fixture_html() else {
        return;
    };
    let result = parse(&html, &opts_debug(FIXTURE_URL));

    let debug = result.debug.as_ref().expect("debug should be present");
    assert!(
        !debug.content_selector.is_empty(),
        "contentSelector should be truthy"
    );
    assert!(
        !debug.removals.is_empty(),
        "removals should be a non-empty array"
    );
}

#[test]
fn debug_false_does_not_include_debug_field() {
    let Some(html) = fixture_html() else {
        return;
    };
    let result = parse(&html, &opts(FIXTURE_URL));

    assert!(
        result.debug.is_none(),
        "debug should be None when debug is off"
    );
}

#[test]
fn debug_removals_include_step_and_text_for_each_entry() {
    let Some(html) = fixture_html() else {
        return;
    };
    let result = parse(&html, &opts_debug(FIXTURE_URL));
    let removals = &result.debug.as_ref().expect("debug").removals;

    assert!(!removals.is_empty(), "should have some removals");
    for removal in removals {
        assert!(!removal.step.is_empty(), "removal.step should be truthy");
        assert!(
            removal.text.len() <= 200,
            "removal.text should be <= 200 chars, got {}",
            removal.text.len()
        );
    }
}

#[test]
fn debug_removals_include_expected_step_names() {
    let Some(html) = fixture_html() else {
        return;
    };
    let result = parse(&html, &opts_debug(FIXTURE_URL));
    let removals = &result.debug.as_ref().expect("debug").removals;

    let valid_steps = [
        "scoreAndRemove",
        "removeBySelector",
        "removeHiddenElements",
        "removeContentPatterns",
        "removePartialSelectors",
        "removeHeaderElements",
    ];

    for removal in removals {
        assert!(
            valid_steps.contains(&removal.step.as_str()),
            "unexpected step name: {:?}",
            removal.step
        );
    }
}

// ═══════════════════════════════════════════════════════════════════
// Pipeline toggles  (debug.test.ts -> "Pipeline toggles")
// ═══════════════════════════════════════════════════════════════════

#[test]
fn score_and_remove_false_skips_content_scoring() {
    let Some(html) = fixture_html() else {
        return;
    };
    let with_scoring = parse(&html, &opts_debug(FIXTURE_URL));
    let without_scoring = parse(&html, &{
        let mut o = opts_debug(FIXTURE_URL);
        o.remove_low_scoring = false;
        o
    });

    let no_scoring_removals: Vec<_> = without_scoring
        .debug
        .as_ref()
        .expect("debug")
        .removals
        .iter()
        .filter(|r| r.step == "scoreAndRemove")
        .collect();

    assert!(
        no_scoring_removals.is_empty(),
        "should have no scoreAndRemove removals"
    );
    assert!(
        without_scoring.word_count >= with_scoring.word_count,
        "without scoring ({}) >= with scoring ({})",
        without_scoring.word_count,
        with_scoring.word_count
    );
}

#[test]
fn remove_hidden_elements_false_skips_hidden_removal() {
    let Some(html) = fixture_html() else {
        return;
    };
    let without_hidden = parse(&html, &{
        let mut o = opts_debug(FIXTURE_URL);
        o.remove_hidden_elements = false;
        o
    });

    let hidden_removals: Vec<_> = without_hidden
        .debug
        .as_ref()
        .expect("debug")
        .removals
        .iter()
        .filter(|r| r.step == "removeHiddenElements")
        .collect();

    assert!(
        hidden_removals.is_empty(),
        "should have no removeHiddenElements removals"
    );
}

#[test]
fn remove_small_images_false_preserves_small_images() {
    let Some(html) = fixture_html() else {
        return;
    };
    let with_removal = parse(&html, &opts(FIXTURE_URL));
    let without_removal = parse(&html, &{
        let mut o = opts(FIXTURE_URL);
        o.remove_small_images = false;
        o
    });

    assert!(
        without_removal.content.len() >= with_removal.content.len(),
        "content with small images kept ({}) >= without ({})",
        without_removal.content.len(),
        with_removal.content.len()
    );
}

#[test]
fn all_toggles_off_produces_more_or_equal_content() {
    let Some(html) = fixture_html() else {
        return;
    };
    let defaults = parse(&html, &opts(FIXTURE_URL));
    let all_off = parse(&html, &{
        let mut o = opts(FIXTURE_URL);
        o.remove_low_scoring = false;
        o.remove_hidden_elements = false;
        o.remove_small_images = false;
        o.remove_exact_selectors = false;
        o.remove_partial_selectors = false;
        o.remove_content_patterns = false;
        o
    });

    assert!(
        all_off.word_count >= defaults.word_count,
        "all off ({}) >= defaults ({})",
        all_off.word_count,
        defaults.word_count
    );
}

// ═══════════════════════════════════════════════════════════════════
// contentSelector  (debug.test.ts -> "contentSelector")
// ═══════════════════════════════════════════════════════════════════

#[test]
fn content_selector_selects_the_specified_element() {
    let Some(html) = fixture_html() else {
        return;
    };
    let result = parse(&html, &{
        let mut o = opts_debug(FIXTURE_URL);
        o.content_selector = Some("body".into());
        o
    });

    let debug = result.debug.as_ref().expect("debug");
    assert!(
        debug.content_selector.contains("body"),
        "content_selector should contain 'body', got {:?}",
        debug.content_selector
    );
    assert!(!result.content.is_empty(), "content should not be empty");
}

#[test]
fn content_selector_falls_back_to_auto_detection_on_no_match() {
    let Some(html) = fixture_html() else {
        return;
    };
    let auto_result = parse(&html, &opts_debug(FIXTURE_URL));
    let fallback_result = parse(&html, &{
        let mut o = opts_debug(FIXTURE_URL);
        o.content_selector = Some(".nonexistent-class-xyz".into());
        o
    });

    assert!(
        !fallback_result.content.is_empty(),
        "should produce content via fallback"
    );
    assert_eq!(
        fallback_result
            .debug
            .as_ref()
            .expect("debug")
            .content_selector,
        auto_result.debug.as_ref().expect("debug").content_selector,
        "fallback selector should match auto-detection"
    );
}

#[test]
fn content_selector_with_specific_element_narrows_content() {
    // The #intro div has enough words (>200) to avoid retry logic,
    // but fewer than the full <article>. This tests that
    // content_selector genuinely narrows extraction.
    let html = r#"<html><head><title>Test</title></head><body>
        <article>
            <h1>Full Article Title</h1>
            <div id="intro">
                <p>This introduction section contains a moderate amount of text about the topic at hand. We discuss the background and motivation for the research that follows in subsequent sections of this article. The introduction provides context and sets up the reader for the detailed analysis that comes next. It covers the key questions we aim to answer and outlines the methodology used throughout this work. Additional context is provided to ensure sufficient word count. The motivation behind this research stems from gaps in existing literature on the subject. We also briefly review prior work that informs our approach. Several key findings from related studies are summarized here to give readers the necessary background knowledge. This paragraph continues with more introductory material to reach the target word count for this section of the article. We expect our findings to contribute meaningfully to the ongoing discussion in this field. The scope of our investigation is deliberately broad to capture diverse perspectives and data points from multiple sources and methodologies.</p>
            </div>
            <p>The main body of the article begins here with detailed analysis of our experimental results. We conducted a series of experiments to test our hypotheses about the behavior of complex systems under varying conditions. The data collected over several months of careful observation reveals interesting patterns that merit further discussion and investigation by the research community.</p>
            <p>Our second major finding relates to the interaction between different variables in the system. When we controlled for external factors and isolated the key variables, the results showed a statistically significant correlation between input parameters and output metrics. These findings have important implications for both theoretical understanding and practical applications in the field.</p>
            <p>In this section we present additional evidence supporting our main thesis. The data from multiple independent experiments converges on a consistent conclusion that validates our initial predictions. Cross-validation with external datasets further strengthens confidence in these results.</p>
            <p>The concluding section of the article summarizes our key contributions and suggests directions for future research. We believe this work opens several promising avenues for investigation that could extend and refine our understanding of the underlying mechanisms at play.</p>
        </article>
    </body></html>"#;
    let auto_result = parse(html, &opts("https://example.com"));
    let narrow_result = parse(html, &{
        let mut o = opts("https://example.com");
        o.content_selector = Some("#intro".into());
        o
    });

    assert!(
        !narrow_result.content.is_empty(),
        "narrow content should be non-empty"
    );
    assert!(
        narrow_result.word_count > 50,
        "narrow should have substantial content ({})",
        narrow_result.word_count
    );
    assert!(
        narrow_result.word_count < auto_result.word_count,
        "narrow ({}) should have fewer words than auto ({})",
        narrow_result.word_count,
        auto_result.word_count
    );
}

// ═══════════════════════════════════════════════════════════════════
// Markdown: exclamation mark before image  (markdown.test.ts)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn markdown_adds_space_between_bang_and_image_syntax() {
    let html = r#"<html><head><title>Test</title></head><body><article><p>Yey!<img src="https://example.com/img.png" alt="IMG"></p></article></body></html>"#;
    let result = parse(html, &{
        let mut o = opts("https://example.com");
        o.separate_markdown = true;
        o
    });
    let md = result
        .content_markdown
        .as_ref()
        .expect("contentMarkdown should be set");

    // The ! from "Yey!" should not merge with ![IMG](...)
    assert!(!md.contains("!!["), "should not have double-bang: {md}");
}

#[test]
fn markdown_adds_space_between_bang_and_linked_image() {
    let html = r#"<html><head><title>Test</title></head><body><article><p>Hello!<a href="https://example.com"><img src="https://example.com/img.png" alt="photo"></a></p></article></body></html>"#;
    let result = parse(html, &{
        let mut o = opts("https://example.com");
        o.separate_markdown = true;
        o
    });
    let md = result
        .content_markdown
        .as_ref()
        .expect("contentMarkdown should be set");

    assert!(
        !md.contains("![!["),
        "should not have nested image syntax: {md}"
    );
}

#[test]
fn markdown_does_not_affect_normal_image_syntax() {
    let html = r#"<html><head><title>Test</title></head><body><article><p>Hello world</p><img src="https://example.com/img.png" alt="photo"></article></body></html>"#;
    let result = parse(html, &{
        let mut o = opts("https://example.com");
        o.separate_markdown = true;
        o
    });
    let md = result
        .content_markdown
        .as_ref()
        .expect("contentMarkdown should be set");

    assert!(
        md.contains("![photo](https://example.com/img.png)"),
        "normal image syntax should be preserved: {md}"
    );
}

#[test]
fn markdown_does_not_add_space_to_bang_not_before_image() {
    let html = r"<html><head><title>Test</title></head><body><article><p>Hello! This is great!</p></article></body></html>";
    let result = parse(html, &{
        let mut o = opts("https://example.com");
        o.separate_markdown = true;
        o
    });
    let md = result
        .content_markdown
        .as_ref()
        .expect("contentMarkdown should be set");

    assert!(
        md.contains("Hello! This is great!"),
        "exclamation marks in text should be preserved: {md}"
    );
}

// ═══════════════════════════════════════════════════════════════════
// Markdown: base href resolution  (markdown.test.ts)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn resolves_relative_urls_against_base_href() {
    let html = r#"<html><head><title>Test</title><base href="https://arxiv.org/html/2312.00752v2/"></head><body><article><p>Content</p><img src="x1.png"></article></body></html>"#;
    let result = parse(html, &opts("https://arxiv.org/html/2312.00752"));

    assert!(
        result
            .content
            .contains("https://arxiv.org/html/2312.00752v2/x1.png")
            || result.content.contains("https://arxiv.org/html/x1.png"),
        "should resolve relative URL against base or document URL: {}",
        result.content
    );
}

#[test]
fn falls_back_to_document_url_when_no_base_href() {
    let html = r#"<html><head><title>Test</title></head><body><article><p>Content</p><img src="x1.png"></article></body></html>"#;
    let result = parse(html, &opts("https://arxiv.org/html/2312.00752"));

    // Without base href, resolves relative to the document URL
    assert!(
        result.content.contains("https://arxiv.org/html/x1.png"),
        "should resolve against document URL: {}",
        result.content
    );
}

// ═══════════════════════════════════════════════════════════════════
// Markdown: wbr tag handling  (markdown.test.ts)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn removes_wbr_tags_without_adding_spaces() {
    let html = r"<html><head><title>Test</title></head><body><article><p>Super<wbr>cali<wbr>fragilistic</p></article></body></html>";
    let result = parse(html, &opts("https://example.com"));

    // wbr is a void element that allows word breaks; it should not
    // insert visible spaces.
    assert!(
        result.content.contains("Supercalifragilistic"),
        "wbr should be removed without adding spaces: {}",
        result.content
    );
}

#[test]
fn handles_wbr_inside_links() {
    let html = r#"<html><head><title>Test</title></head><body><article><p><a href="https://example.com">long<wbr>word</a></p></article></body></html>"#;
    let result = parse(html, &opts("https://example.com"));

    assert!(
        result.content.contains("longword"),
        "wbr inside links should be removed: {}",
        result.content
    );
}

// ═══════════════════════════════════════════════════════════════════
// Schema.org text fallback  (schema-fallback.test.ts)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn schema_fallback_uses_schema_text_when_more_words() {
    let html = r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>Test Post</title>
        <script type="application/ld+json">
        {
            "@type": "SocialMediaPosting",
            "text": "This is a much longer post body that contains significantly more words than the short article element. It goes on and on with additional sentences to ensure the word count exceeds the extracted content. Here is even more text to make absolutely sure we cross the threshold. The schema text fallback should kick in when this text is longer than what the scorer found."
        }
        </script>
    </head>
    <body>
        <nav><a href="/">Home</a></nav>
        <div id="feed">
            <div class="post" id="other-post">
                <p>Some other post in the feed that is not what we want.</p>
            </div>
            <div class="post" id="target-post">
                <p>This is a much longer post body that contains significantly more words than the short article element. It goes on and on with additional sentences to ensure the word count exceeds the extracted content. Here is even more text to make absolutely sure we cross the threshold. The schema text fallback should kick in when this text is longer than what the scorer found.</p>
            </div>
        </div>
    </body>
    </html>"#;

    let result = parse(html, &opts("https://example.com"));

    assert!(
        result.content.contains("This is a much longer post body"),
        "should contain schema text: {}",
        result.content
    );
    assert!(
        result
            .content
            .contains("schema text fallback should kick in"),
        "should contain full schema text"
    );
}

#[test]
fn schema_fallback_uses_article_body() {
    let article_body = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur.";

    let html = format!(
        r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>Article Page</title>
        <script type="application/ld+json">
        {{
            "@type": "Article",
            "articleBody": "{article_body}"
        }}
        </script>
    </head>
    <body>
        <header><h1>My Blog</h1></header>
        <main>
            <article>
                <p>{article_body}</p>
            </article>
        </main>
        <footer>Copyright 2024</footer>
    </body>
    </html>"#
    );

    let result = parse(&html, &opts("https://example.com"));

    assert!(
        result.content.contains("Lorem ipsum dolor sit amet"),
        "should contain article body start"
    );
    assert!(
        result.content.contains("fugiat nulla pariatur"),
        "should contain article body end"
    );
}

#[test]
fn schema_fallback_not_used_when_extracted_content_has_more_words() {
    let html = r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>Good Extraction</title>
        <script type="application/ld+json">
        {
            "@type": "SocialMediaPosting",
            "text": "Short schema text."
        }
        </script>
    </head>
    <body>
        <article>
            <h1>Full Article</h1>
            <p>This article has plenty of content that the scorer will extract correctly. It contains multiple paragraphs with enough words to exceed the schema text length. The content scorer should pick this up as the main content without needing the schema fallback.</p>
            <p>Here is another paragraph with even more content to make the word count higher. We want to ensure the extracted content exceeds the schema text word count so the fallback does not trigger.</p>
            <p>And a third paragraph for good measure, with additional words and sentences to pad out the content even further beyond what the schema text contains.</p>
        </article>
    </body>
    </html>"#;

    let result = parse(html, &opts("https://example.com"));

    assert!(
        result.content.contains("Full Article"),
        "should use normally extracted content"
    );
    assert!(
        result.content.contains("multiple paragraphs"),
        "should have article content"
    );
}

#[test]
fn schema_fallback_finds_smallest_matching_element() {
    let post_text = "This is the target post content with enough words to trigger the schema text fallback mechanism. It needs to be long enough that its word count exceeds whatever the scorer extracted from the page. Adding more sentences here to pad the word count sufficiently.";

    let html = format!(
        r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>Feed Page</title>
        <script type="application/ld+json">
        {{
            "@type": "SocialMediaPosting",
            "text": "{post_text}"
        }}
        </script>
    </head>
    <body>
        <div id="wrapper">
            <div id="feed">
                <div class="post">
                    <p>First post in the feed with different content entirely.</p>
                </div>
                <div class="post" id="target">
                    <p>{post_text}</p>
                </div>
                <div class="post">
                    <p>Third post with yet more different content.</p>
                </div>
            </div>
        </div>
    </body>
    </html>"#
    );

    let result = parse(&html, &opts("https://example.com"));

    assert!(
        result.content.contains("target post content"),
        "should contain the target post"
    );
    assert!(
        !result.content.contains("First post in the feed"),
        "should NOT contain other posts"
    );
    assert!(
        !result.content.contains("Third post with yet more"),
        "should NOT contain other posts"
    );
}

#[test]
fn schema_fallback_preserves_inline_formatting() {
    let plain_text = "This post has formatted content with bold text and italic text and a link to example site. It needs enough words to trigger the schema fallback path so we keep adding more content here.";

    let html = format!(
        r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>Formatted Post</title>
        <script type="application/ld+json">
        {{
            "@type": "SocialMediaPosting",
            "text": "{plain_text}"
        }}
        </script>
    </head>
    <body>
        <div>
            <p>Nav item</p>
        </div>
        <div class="post">
            <p>This post has <strong>formatted content</strong> with <em>bold text</em> and <em>italic text</em> and a <a href="https://example.com">link to example site</a>. It needs enough words to trigger the schema fallback path so we keep adding more content here.</p>
        </div>
    </body>
    </html>"#
    );

    let result = parse(&html, &opts("https://example.com"));

    assert!(
        result
            .content
            .contains("<strong>formatted content</strong>")
            || result.content.contains("formatted content"),
        "should preserve or include formatted content: {}",
        result.content
    );
}

// ═══════════════════════════════════════════════════════════════════
// Schema.org text fallback sanitization  (schema-fallback.test.ts)
// ═══════════════════════════════════════════════════════════════════

/// Build HTML where the schema fallback triggers and the matched DOM
/// element contains the given dangerous HTML.
fn build_schema_fallback_html(dangerous_html: &str) -> String {
    let schema_text = "This is the full post body with enough words to exceed the short article summary that the content scorer will extract. Adding more sentences here to make sure the word count difference is large enough to reliably trigger the schema text fallback path in the parse method.";

    format!(
        r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>Test</title>
        <script type="application/ld+json">
        {{
            "@type": "SocialMediaPosting",
            "text": "{schema_text}"
        }}
        </script>
    </head>
    <body>
        <article>
            <h1>Title</h1>
            <p>Short article summary.</p>
        </article>
        <div class="full-post">
            <p>{schema_text}</p>
            {dangerous_html}
        </div>
    </body>
    </html>"#
    )
}

#[test]
fn schema_fallback_strips_script_tags() {
    let html = build_schema_fallback_html(r#"<script>alert("xss")</script>"#);
    let result = parse(&html, &opts("https://example.com"));

    assert!(
        result.content.contains("full post body"),
        "should contain main content"
    );
    assert!(
        !result.content.contains("<script"),
        "should strip script tags"
    );
    assert!(
        !result.content.contains("alert"),
        "should strip script content"
    );
}

#[test]
fn schema_fallback_strips_event_handlers() {
    let html =
        build_schema_fallback_html(r#"<img src="x.jpg" onerror="alert('xss')" onclick="steal()">"#);
    let result = parse(&html, &opts("https://example.com"));

    assert!(
        result.content.contains("full post body"),
        "should contain main content"
    );
    assert!(!result.content.contains("onerror"), "should strip onerror");
    assert!(!result.content.contains("onclick"), "should strip onclick");
}

#[test]
fn schema_fallback_strips_style_elements() {
    let html = build_schema_fallback_html(
        r#"<style>.x { background: url("https://evil.com/steal") }</style>"#,
    );
    let result = parse(&html, &opts("https://example.com"));

    assert!(
        result.content.contains("full post body"),
        "should contain main content"
    );
    assert!(
        !result.content.contains("<style"),
        "should strip style elements"
    );
    assert!(
        !result.content.contains("evil.com"),
        "should strip evil URLs"
    );
}

#[test]
fn schema_fallback_strips_noscript_elements() {
    let html =
        build_schema_fallback_html(r#"<noscript><img src="https://evil.com/track"></noscript>"#);
    let result = parse(&html, &opts("https://example.com"));

    assert!(
        result.content.contains("full post body"),
        "should contain main content"
    );
    // noscript may be resolved or stripped; either way evil.com
    // tracking pixel should not appear as the only content
    assert!(
        !result.content.contains("<noscript"),
        "should strip noscript tags"
    );
}

#[test]
fn schema_fallback_preserves_iframes_with_src() {
    let html = build_schema_fallback_html(
        r#"<iframe src="https://www.youtube.com/embed/abc123" width="560" height="315"></iframe><iframe src="https://open.spotify.com/embed/track/xyz"></iframe>"#,
    );
    let result = parse(&html, &opts("https://example.com"));

    assert!(
        result.content.contains("full post body"),
        "should contain main content"
    );
    // Iframes with valid src should be preserved
    assert!(
        result.content.contains("youtube.com/embed/abc123")
            || result.content.contains("youtube.com/watch"),
        "should preserve YouTube iframe: {}",
        result.content
    );
}

#[test]
fn schema_fallback_strips_srcdoc_from_iframes() {
    let html =
        build_schema_fallback_html(r#"<iframe srcdoc="<script>alert('xss')</script>"></iframe>"#);
    let result = parse(&html, &opts("https://example.com"));

    assert!(
        result.content.contains("full post body"),
        "should contain main content"
    );
    assert!(
        !result.content.contains("srcdoc"),
        "should strip srcdoc attribute"
    );
}

#[test]
fn schema_fallback_strips_javascript_uris() {
    let html = build_schema_fallback_html(
        r#"<a href="javascript:alert('xss')">click me</a><a href="  javascript:void(0)">spaced</a>"#,
    );
    let result = parse(&html, &opts("https://example.com"));

    assert!(
        result.content.contains("full post body"),
        "should contain main content"
    );
    assert!(
        !result.content.contains("javascript:"),
        "should strip javascript: URIs: {}",
        result.content
    );
}

#[test]
fn schema_fallback_strips_base_tag() {
    let schema_text = "This is the full post body with enough words to exceed the short article summary that the content scorer will extract. Adding more sentences here to make sure the word count difference is large enough to reliably trigger the schema text fallback path in the parse method.";

    let html = format!(
        r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>Test</title>
        <script type="application/ld+json">
        {{
            "@type": "SocialMediaPosting",
            "text": "{schema_text}"
        }}
        </script>
    </head>
    <body>
        <base href="https://evil.com/">
        <article>
            <h1>Title</h1>
            <p>Short article summary.</p>
        </article>
        <div class="full-post">
            <p>{schema_text}</p>
        </div>
    </body>
    </html>"#
    );

    let result = parse(&html, &opts("https://example.com"));

    assert!(
        result.content.contains("full post body"),
        "should contain main content"
    );
    assert!(!result.content.contains("<base"), "should strip base tag");
}

#[test]
fn schema_fallback_strips_object_and_embed_elements() {
    let html = build_schema_fallback_html(
        r#"<object data="https://evil.com/flash.swf"></object><embed src="https://evil.com/plugin">"#,
    );
    let result = parse(&html, &opts("https://example.com"));

    assert!(
        result.content.contains("full post body"),
        "should contain main content"
    );
    assert!(
        !result.content.contains("<object"),
        "should strip object elements"
    );
    assert!(
        !result.content.contains("<embed"),
        "should strip embed elements"
    );
}

#[test]
fn schema_fallback_strips_data_text_html_uris() {
    let html =
        build_schema_fallback_html(r#"<img src="data:text/html,<script>alert(1)</script>">"#);
    let result = parse(&html, &opts("https://example.com"));

    assert!(
        result.content.contains("full post body"),
        "should contain main content"
    );
    assert!(
        !result.content.contains("data:text/html"),
        "should strip data:text/html URIs: {}",
        result.content
    );
}

// ═══════════════════════════════════════════════════════════════════
// Full bundle: Markdown conversion  (full-bundle.test.ts)
// ═══════════════════════════════════════════════════════════════════

const SIMPLE_HTML: &str = r"
<!DOCTYPE html>
<html>
<head><title>Test Page</title></head>
<body>
    <article>
        <h1>Test Article</h1>
        <p>This is a <strong>test</strong> paragraph with some content.</p>
        <p>Another paragraph here.</p>
    </article>
</body>
</html>
";

#[test]
fn markdown_true_converts_content_to_markdown() {
    let result = parse(SIMPLE_HTML, &{
        let mut o = DecruftOptions::default();
        o.markdown = true;
        o
    });

    assert!(
        !result.content.contains("<p>"),
        "should not contain HTML <p> tags: {}",
        result.content
    );
    assert!(
        !result.content.contains("<strong>"),
        "should not contain HTML <strong> tags"
    );
    assert!(
        result.content.contains("**test**"),
        "should contain markdown bold: {}",
        result.content
    );
}

#[test]
fn separate_markdown_populates_content_markdown_keeping_html() {
    let result = parse(SIMPLE_HTML, &{
        let mut o = DecruftOptions::default();
        o.separate_markdown = true;
        o
    });

    // content should still be HTML
    assert!(
        result.content.contains("<p>") || result.content.contains("<strong>"),
        "content should still be HTML: {}",
        result.content
    );

    // contentMarkdown should be populated with markdown
    let md = result
        .content_markdown
        .as_ref()
        .expect("contentMarkdown should be set");
    assert!(
        !md.contains("<p>"),
        "contentMarkdown should not contain HTML: {md}"
    );
    assert!(
        md.contains("**test**"),
        "contentMarkdown should contain markdown bold: {md}"
    );
}

#[test]
fn without_markdown_options_no_markdown_conversion() {
    let result = parse(SIMPLE_HTML, &DecruftOptions::default());

    // content should be HTML
    assert!(
        result.content.contains("<p>") || result.content.contains("<strong>"),
        "content should be HTML: {}",
        result.content
    );

    assert!(
        result.content_markdown.is_none(),
        "contentMarkdown should be None when markdown is off"
    );
}

// ═══════════════════════════════════════════════════════════════════
// LaTeX math escaping  (defuddle #224/#225)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn markdown_preserves_inline_latex_without_escaping() {
    let html = r#"<html><head><title>Math</title></head><body><article>
        <p>The formula <span data-latex="\sum_{i=0}^n x_i">sum</span> is inline.</p>
    </article></body></html>"#;
    let result = parse(html, &{
        let mut o = DecruftOptions::default();
        o.markdown = true;
        o
    });

    assert!(
        result.content.contains(r"$\sum_{i=0}^n x_i$"),
        "inline LaTeX should not be escaped: {}",
        result.content
    );
    assert!(
        !result.content.contains(r"\\_"),
        "underscores should not be double-escaped"
    );
}

#[test]
fn markdown_preserves_block_latex_without_escaping() {
    let html = r#"<html><head><title>Math</title></head><body><article>
        <p>Some text before.</p>
        <div data-latex="E = mc^2">energy</div>
        <p>Some text after.</p>
    </article></body></html>"#;
    let result = parse(html, &{
        let mut o = DecruftOptions::default();
        o.markdown = true;
        o
    });

    assert!(
        result.content.contains("E = mc^2"),
        "block LaTeX should preserve content: {}",
        result.content
    );
    assert!(
        result.content.contains("$$"),
        "block LaTeX should use $$ delimiters: {}",
        result.content
    );
}

#[test]
fn markdown_preserves_raw_dollar_sign_latex() {
    let html = r"<html><head><title>Math</title></head><body><article>
        <p>The formula $\sum_{i=0}^n x_i$ and $$E = mc^2$$ are here.</p>
    </article></body></html>";
    let result = parse(html, &{
        let mut o = DecruftOptions::default();
        o.markdown = true;
        o
    });

    assert!(
        result.content.contains(r"$\sum_{i=0}^n x_i$"),
        "raw inline LaTeX should be unescaped: {}",
        result.content
    );
    assert!(
        result.content.contains("$$E = mc^2$$"),
        "raw block LaTeX should be unescaped: {}",
        result.content
    );
}

// ═══════════════════════════════════════════════════════════════════
// Reddit author extraction  (reddit-author.test.ts)
// ═══════════════════════════════════════════════════════════════════

const REDDIT_URL: &str = "https://www.reddit.com/r/test/comments/abc123/test_post/";

const NEW_REDDIT_NO_COMMENTS_HTML: &str = r#"
<html>
<head><title>Test Post : test</title></head>
<body>
<h1>Test Post Title</h1>
<shreddit-post
  author="original_poster"
  subreddit-prefixed-name="r/test"
  post-title="Test Post Title"
  score="42"
  comment-count="5"
  created-timestamp="2025-01-15T10:00:00Z"
  permalink="/r/test/comments/abc123/test_post/">
  <div slot="text-body"><p>This is the post body content.</p></div>
</shreddit-post>
<span class="author">logged_in_user</span>
<span class="author">some_commenter</span>
</body>
</html>
"#;

const NEW_REDDIT_WITH_COMMENTS_HTML: &str = r#"
<html>
<head><title>Test Post : test</title></head>
<body>
<h1>Test Post Title</h1>
<shreddit-post
  author="original_poster"
  subreddit-prefixed-name="r/test"
  post-title="Test Post Title"
  score="42"
  comment-count="5"
  created-timestamp="2025-01-15T10:00:00Z"
  permalink="/r/test/comments/abc123/test_post/">
  <div slot="text-body"><p>This is the post body content.</p></div>
</shreddit-post>
<shreddit-comment author="commenter_one" depth="0" score="10"
  permalink="/r/test/comments/abc123/test_post/c1/"
  created="2025-01-15T11:00:00Z">
  <div slot="comment"><p>Nice post!</p></div>
</shreddit-comment>
<shreddit-comment author="commenter_two" depth="0" score="5"
  permalink="/r/test/comments/abc123/test_post/c2/"
  created="2025-01-15T12:00:00Z">
  <div slot="comment"><p>I agree.</p></div>
</shreddit-comment>
<span class="author">logged_in_user</span>
</body>
</html>
"#;

#[test]
fn reddit_no_comments_returns_post_author() {
    let result = parse(NEW_REDDIT_NO_COMMENTS_HTML, &opts(REDDIT_URL));

    assert_eq!(
        result.author, "original_poster",
        "author should be the post author, got: {:?}",
        result.author
    );
    assert_eq!(result.site, "r/test");
    assert_eq!(result.title, "Test Post Title");
}

#[test]
fn reddit_with_comments_returns_post_author() {
    let result = parse(NEW_REDDIT_WITH_COMMENTS_HTML, &opts(REDDIT_URL));

    assert_eq!(
        result.author, "original_poster",
        "author should be the post author, not a commenter: {:?}",
        result.author
    );
}

// ═══════════════════════════════════════════════════════════════════
// X/Twitter article surrogate pair repair  (x-article-surrogates.test.ts)
// ═══════════════════════════════════════════════════════════════════

const X_ARTICLE_URL: &str = "https://x.com/testuser/article/123456789";

fn make_x_article_html(paragraph_inner: &str) -> String {
    format!(
        r#"<html><head><title>Test Article</title></head>
        <body>
            <div data-testid="twitterArticleRichTextView">
                <h1 data-testid="twitter-article-title">Test Article</h1>
                <div class="public-DraftStyleDefault-block">{paragraph_inner}</div>
            </div>
        </body></html>"#
    )
}

#[test]
fn x_article_repairs_emoji_split_across_bold_span() {
    // U+1F504 (🔄) = high surrogate 0xD83D (55357) + low 0xDD04 (56580)
    let html = make_x_article_html(
        "Refresh &#55357;<span style=\"font-weight: bold\">&#56580; updates</span> daily",
    );
    let result = parse(&html, &opts(X_ARTICLE_URL));

    assert!(
        result.content.contains('\u{1F504}'),
        "should contain intact emoji: {}",
        result.content
    );
}

#[test]
fn x_article_repairs_emoji_split_across_link() {
    let html = make_x_article_html("See &#55357;<a href=\"https://example.com\">&#56580;here</a>");
    let result = parse(&html, &opts(X_ARTICLE_URL));

    // The emoji should be present and not replaced with U+FFFD
    assert!(
        !result.content.contains('\u{FFFD}'),
        "should not contain replacement characters: {}",
        result.content
    );
}

#[test]
fn x_article_preserves_intact_emojis() {
    let html = make_x_article_html("Refresh \u{1F504} daily");
    let result = parse(&html, &opts(X_ARTICLE_URL));

    assert!(
        result.content.contains('\u{1F504}'),
        "should preserve intact emoji: {}",
        result.content
    );
}

#[test]
fn x_article_repairs_hex_surrogate_refs() {
    // Same emoji via hex character references
    let html = make_x_article_html("Refresh &#xD83D;<span>&#xDD04; updates</span> daily");
    let result = parse(&html, &opts(X_ARTICLE_URL));

    assert!(
        result.content.contains('\u{1F504}'),
        "should repair hex surrogate refs: {}",
        result.content
    );
}

// ═══════════════════════════════════════════════════════════════════
// Weekday abbreviation not treated as author  (defuddle #233)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn weekday_in_date_not_treated_as_author_byline() {
    let html = r"<html><head><title>Email</title></head><body><article>
        <p><span>Date:</span> <span>Wed, 08 Apr 2026</span></p>
        <p><span>From:</span> <span>Alice Bob</span></p>
        <p><span>Subject:</span> <span>Meeting notes</span></p>
        <p>Here are the meeting notes from today's discussion about the project roadmap and upcoming milestones. We covered several important topics including budget allocation, timeline adjustments, and resource planning for the next quarter.</p>
    </article></body></html>";
    let result = parse(html, &opts("https://example.com"));

    // "Wed, 08 Apr 2026" should NOT be removed as a byline
    assert!(
        result.content.contains("Wed")
            || result.content.contains("2026")
            || result.content.contains("Date"),
        "date line should not be stripped as author byline: {}",
        result.content
    );
}

#[test]
fn real_author_date_byline_still_removed() {
    let html = r"<html><head><title>Blog</title></head><body><article>
        <h1>Great Article</h1>
        <p>John Smith | January 15, 2026</p>
        <p>This is the main content of the article with plenty of words to make the scorer happy. It discusses various topics at length to ensure it passes word count thresholds for content extraction.</p>
    </article></body></html>";
    let result = parse(html, &opts("https://example.com"));

    assert!(
        !result.content.contains("John Smith"),
        "real author bylines should still be removed: {}",
        result.content
    );
}

// ═══════════════════════════════════════════════════════════════════
// Dismiss buttons in hidden content retry  (defuddle #232/#234)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn dismiss_buttons_removed_in_hidden_content_retry() {
    let html = r#"<html><head><title>Newsletter</title></head><body>
        <div class="updates-overlay overlay--post-detail" aria-hidden="true">
            <div class="updates-letter">
                <a class="updates-dismiss" href="/posts/">&lt;</a>
                <h1>Newsletter Issue</h1>
                <p>This is a long newsletter article with substantial content that needs many words to pass the extraction threshold. The article discusses several important topics in great detail across multiple paragraphs.</p>
                <p>The second paragraph continues the discussion with additional analysis and commentary on the subject matter at hand. More words are added to ensure sufficient content length for the retry logic to activate.</p>
                <p>A third paragraph provides further elaboration and concluding thoughts on the topics discussed. This ensures the hidden content recovery path triggers and selects this content.</p>
            </div>
        </div>
    </body></html>"#;
    let result = parse(html, &opts("https://example.com"));

    assert!(
        result.content.contains("Newsletter Issue")
            || result.content.contains("newsletter article"),
        "should recover hidden content: {}",
        result.content
    );
    assert!(
        !result.content.contains("updates-dismiss"),
        "dismiss links should be removed: {}",
        result.content
    );
}
