//! Content extraction correctness tests.
//!
//! Three levels:
//! 1. Fixture sweeps — all 144 defuddle + 130 mozilla fixtures produce non-empty output
//! 2. Metadata validation — field-by-field checks on key fixtures
//! 3. Content assertions — "with/without" checks on hand-picked fixtures
//!
//! This file does NOT test exact output (that's regression.rs) or option
//! toggles (that's behavior.rs). It tests whether the right content is
//! extracted with default settings.

#![allow(clippy::panic)]

use decruft::{DecruftOptions, parse};
use std::fs;
use std::path::PathBuf;

fn load(name: &str) -> String {
    let path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

fn load_defuddle(name: &str) -> String {
    load(&format!("defuddle/{name}"))
}

fn opts(url: &str) -> DecruftOptions {
    let mut o = DecruftOptions::default();
    o.url = Some(url.into());
    o
}

fn url_from_html(html: &str) -> Option<String> {
    let start = html.find("<!-- {")?;
    let json_start = start + 5;
    let end = html[json_start..].find(" -->")?;
    let json_str = &html[json_start..json_start + end];
    serde_json::from_str::<serde_json::Value>(json_str)
        .ok()?
        .get("url")
        .and_then(|v| v.as_str())
        .map(String::from)
}

// ════════════════════════════════════════════════════════════════
// 1. Fixture sweeps
// ════════════════════════════════════════════════════════════════

#[test]
fn defuddle_all_extract_content() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/defuddle");
    let mut failures = Vec::new();
    let mut total = 0;

    for entry in fs::read_dir(&dir).unwrap().flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("html") {
            continue;
        }
        total += 1;
        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        let html = fs::read_to_string(&path).unwrap();
        let url = url_from_html(&html).unwrap_or_else(|| format!("https://example.com/{name}"));
        let result = parse(&html, &opts(&url));

        if result.content.trim().is_empty() || result.word_count == 0 {
            failures.push(name);
        }
    }

    assert!(total >= 100, "expected ≥100 fixtures, got {total}");
    assert!(failures.is_empty(), "empty extractions: {failures:?}");
}

#[test]
fn mozilla_all_extract_content() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/mozilla");
    let mut failures = Vec::new();
    let mut total = 0;

    for entry in fs::read_dir(&dir).unwrap().flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        let source = path.join("source.html");
        let Ok(html) = fs::read_to_string(&source) else {
            continue;
        };

        // Skip fixtures flagged as not readerable
        let meta_path = path.join("expected-metadata.json");
        if let Ok(meta_str) = fs::read_to_string(&meta_path) {
            if let Ok(meta) = serde_json::from_str::<serde_json::Value>(&meta_str) {
                if meta.get("readerable").and_then(serde_json::Value::as_bool) == Some(false) {
                    continue;
                }
            }
        }

        total += 1;
        let result = parse(&html, &DecruftOptions::default());
        if result.word_count == 0 {
            failures.push(name);
        }
    }

    assert!(total >= 100, "expected ≥100 fixtures, got {total}");
    assert!(failures.is_empty(), "empty extractions: {failures:?}");
}

// ════════════════════════════════════════════════════════════════
// 2. Metadata validation
// ════════════════════════════════════════════════════════════════

/// (fixture, title, author, published prefix)
const METADATA_CHECKS: &[(&str, &str, &str, &str)] = &[
    (
        "general--stephango.com-buy-wisely.html",
        "Buy wisely",
        "Steph Ango",
        "2023-09-30",
    ),
    ("general--wikipedia.html", "Obsidian (software)", "", ""),
    (
        "general--appendix-heading.html",
        "Article with Appendix",
        "",
        "",
    ),
    (
        "general--back-nav-link.html",
        "An Article About Sorting",
        "",
        "",
    ),
    (
        "general--trailing-cta-newsletter.html",
        "New Feature Announcement",
        "Engineering Team",
        "",
    ),
    (
        "general--inline-comments-and-link-lists.html",
        "New Product Announced with Major Upgrades",
        "Jane Smith",
        "",
    ),
    (
        "content-patterns--card-grid-stripped-headings.html",
        "How Spacecraft Plumbing Works",
        "Jane Smith",
        "",
    ),
    (
        "content-patterns--trailing-related-posts.html",
        "Coffee Cooling Article",
        "",
        "",
    ),
    (
        "general--daringfireball.net-2025-02-the_iphone_16e.html",
        "The iPhone 16e",
        "",
        "",
    ),
    (
        "general--cp4space-jordan-algebra.html",
        "The exceptional Jordan algebra",
        "apgoucher",
        "2020-10-28",
    ),
];

#[test]
fn defuddle_metadata() {
    let mut failures = Vec::new();

    for &(file, title, author, published) in METADATA_CHECKS {
        let html = load_defuddle(file);
        let name = file.strip_suffix(".html").unwrap_or(file);
        let url = url_from_html(&html).unwrap_or_else(|| format!("https://example.com/{name}"));
        let result = parse(&html, &opts(&url));

        if result.title != title {
            failures.push(format!("{name}: title {title:?} != {:?}", result.title));
        }
        if result.author != author {
            failures.push(format!("{name}: author {author:?} != {:?}", result.author));
        }
        if !published.is_empty() && !result.published.contains(published) {
            failures.push(format!(
                "{name}: published {:?} missing {published:?}",
                result.published
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "metadata mismatches:\n  {}",
        failures.join("\n  ")
    );
}

#[test]
fn mozilla_title_match_rate() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/mozilla");
    let mut matches = 0;
    let mut total = 0;

    for entry in fs::read_dir(&dir).unwrap().flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let meta_path = path.join("expected-metadata.json");
        let Ok(meta_str) = fs::read_to_string(&meta_path) else {
            continue;
        };
        let Ok(meta) = serde_json::from_str::<serde_json::Value>(&meta_str) else {
            continue;
        };
        let Some(expected) = meta.get("title").and_then(|v| v.as_str()) else {
            continue;
        };
        if expected.is_empty() {
            continue;
        }

        let source = path.join("source.html");
        let Ok(html) = fs::read_to_string(&source) else {
            continue;
        };

        total += 1;
        let result = parse(&html, &DecruftOptions::default());
        if result.title == expected
            || result.title.contains(expected)
            || expected.contains(&result.title)
        {
            matches += 1;
        }
    }

    #[allow(clippy::cast_precision_loss)]
    let pct = (matches * 100) / total.max(1);
    assert!(pct >= 80, "title match {matches}/{total} ({pct}%) < 80%");
}

// ════════════════════════════════════════════════════════════════
// 3. Content assertions (with/without)
// ════════════════════════════════════════════════════════════════

/// Assert content contains all `with` strings and none of the `without` strings.
fn check_content(html: &str, url: &str, with: &[&str], without: &[&str]) {
    let result = parse(html, &opts(url));
    for phrase in with {
        assert!(
            result.content.contains(phrase),
            "missing expected phrase {phrase:?} in extraction from {url}"
        );
    }
    for phrase in without {
        assert!(
            !result.content.contains(phrase),
            "found unwanted phrase {phrase:?} in extraction from {url}"
        );
    }
}

// ── Blog post ───────────────────────────────────────────────────

#[test]
fn blog_content_preserved() {
    let html = load("complex_blog.html");
    check_content(
        &html,
        "https://example.com/article",
        &[
            "Rust's ownership system",
            "Borrowing and References",
            "Lifetimes",
            "String::from",
        ],
        &[],
    );
}

#[test]
fn blog_clutter_removed() {
    let html = load("complex_blog.html");
    check_content(
        &html,
        "https://example.com/article",
        &[],
        &[
            "main-nav",
            "ad-banner",
            "sidebar-ad",
            "Popular Posts",
            "cookie",
            "Privacy Policy",
            "All rights reserved",
            "share-twitter",
            "Comments (12)",
            "newsletter",
            "You Might Also Like",
            "promo-popup",
            "pixel.gif",
        ],
    );
}

#[test]
fn blog_metadata() {
    let html = load("complex_blog.html");
    let result = parse(&html, &opts("https://example.com/article"));
    assert_eq!(result.title, "Understanding Rust Ownership");
    assert_eq!(result.author, "Alice Chen");
    assert_eq!(result.site, "TechBlog Daily");
    assert!(result.published.contains("2025-03-15"));
    assert!(result.word_count > 200 && result.word_count < 1000);
}

// ── News article ────────────────────────────────────────────────

#[test]
fn news_content_preserved() {
    let html = load("news_article.html");
    check_content(
        &html,
        "https://example.com/article",
        &[
            "Marine biologists",
            "Aurelia profundis",
            "chemosynthetic",
            "<blockquote",
            "<figure",
        ],
        &[],
    );
}

#[test]
fn news_clutter_removed() {
    let html = load("news_article.html");
    check_content(
        &html,
        "https://example.com/article",
        &[],
        &[
            "inline-ad",
            "ADVERTISEMENT",
            "Trending Now",
            "Sponsored Stories",
            "BREAKING",
            "More Science Stories",
            "Enable notifications",
        ],
    );
}

#[test]
fn news_metadata() {
    let html = load("news_article.html");
    let result = parse(&html, &opts("https://example.com/article"));
    assert_eq!(
        result.title,
        "Scientists Discover New Species in Deep Ocean"
    );
    assert_eq!(result.author, "Sarah Mitchell");
    assert_eq!(result.site, "World News Today");
    assert!(result.published.contains("2025-06-20"));
}

// ── Defuddle key fixtures ───────────────────────────────────────

#[test]
fn stephango_content() {
    let html = load_defuddle("general--stephango.com-buy-wisely.html");
    check_content(
        &html,
        "https://stephango.com/buy-wisely",
        &["cost per use", "Darn Tough"],
        &[],
    );
    let result = parse(&html, &opts("https://stephango.com/buy-wisely"));
    assert!(result.word_count > 500);
}

#[test]
fn wikipedia_content() {
    let html = load_defuddle("general--wikipedia.html");
    check_content(
        &html,
        "https://en.wikipedia.org/wiki/Obsidian",
        &["personal knowledge management"],
        &[],
    );
    let result = parse(&html, &opts("https://en.wikipedia.org/wiki/Obsidian"));
    assert!(result.word_count > 200);
}

#[test]
fn card_grid_removes_cards() {
    let html = load_defuddle("content-patterns--card-grid-stripped-headings.html");
    check_content(
        &html,
        "https://example.com/spacecraft",
        &["microgravity"],
        &["Ion Thruster Breaks"],
    );
}

#[test]
fn appendix_preserved() {
    let html = load_defuddle("general--appendix-heading.html");
    check_content(
        &html,
        "https://example.com/appendix",
        &["Appendix", "main article content"],
        &[],
    );
}

// ── Bug fix regressions ─────────────────────────────────────────

#[test]
fn stripe_code_blocks_preserved() {
    // #8: "dropdown" partial pattern was stripping CodeTabGroup containers
    let html = load_defuddle("codeblocks--stripe.html");
    check_content(
        &html,
        "https://stripe.com/docs",
        &["paymentMiddleware", "curl"],
        &[],
    );
}

#[test]
fn scp_wiki_footnotes_preserved() {
    // #7: "footer" partial pattern was stripping footnotes-footer containers
    let html = load_defuddle("general--scp-wiki.wikidot.com-scp-9935.html");
    check_content(
        &html,
        "https://scp-wiki.wikidot.com/scp-9935",
        &["No relation to the Washington Nationals"],
        &[],
    );
}

#[test]
fn cp4space_title_and_bibliography_preserved() {
    // #10: "entry-title" partial removed article heading; trailing links removed bibliography
    let html = load_defuddle("general--cp4space-jordan-algebra.html");
    check_content(
        &html,
        "https://cp4space.hatsya.com/2020/10/28/the-exceptional-jordan-algebra/",
        &["exceptional Jordan algebra", "John Baez"],
        &[],
    );
}

// ── Edge cases ──────────────────────────────────────────────────

#[test]
fn empty_document() {
    let result = parse("", &DecruftOptions::default());
    assert!(result.content.is_empty() || result.word_count == 0);
}

#[test]
fn minimal_document() {
    let result = parse(
        "<html><body><p>Hello world</p></body></html>",
        &DecruftOptions::default(),
    );
    assert!(result.content.contains("Hello world"));
}

#[test]
fn navigation_only() {
    let html = r#"<html><body>
        <nav><ul><li><a href="/">Home</a></li></ul></nav>
        <footer><p>Copyright 2025</p></footer>
    </body></html>"#;
    let result = parse(html, &DecruftOptions::default());
    assert!(result.word_count < 10);
}

#[test]
fn multiple_articles_picks_main() {
    let html = r"<html><body>
        <article>
            <h1>Main Article</h1>
            <p>Substantial content that should be the primary target because it has many more words.</p>
            <p>Additional paragraph with more meaningful content about the topic.</p>
        </article>
        <aside><article><h2>Sidebar</h2><p>Short</p></article></aside>
    </body></html>";
    let result = parse(html, &DecruftOptions::default());
    assert!(result.content.contains("Main Article"));
}

#[test]
fn semantic_html_preserved() {
    let html = r"<html><body><article>
        <h1>Title</h1>
        <p><strong>bold</strong> and <em>italic</em></p>
        <ul><li>Item</li></ul>
        <table><tr><th>H</th></tr><tr><td>D</td></tr></table>
    </article></body></html>";
    let result = parse(html, &DecruftOptions::default());
    assert!(result.content.contains("<strong>bold</strong>"));
    assert!(result.content.contains("<em>italic</em>"));
    assert!(result.content.contains("<ul>"));
    assert!(result.content.contains("<table>"));
}

// ── Mozilla individual sites (minimum word counts) ──────────────

macro_rules! mozilla_site {
    ($name:ident, $fixture:expr, $min_words:expr) => {
        #[test]
        fn $name() {
            let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/mozilla")
                .join($fixture);
            let html = fs::read_to_string(dir.join("source.html"))
                .unwrap_or_else(|e| panic!("{}: {e}", $fixture));
            let result = parse(&html, &DecruftOptions::default());
            assert!(
                result.word_count >= $min_words,
                "{}: {} words < {}",
                $fixture,
                result.word_count,
                $min_words
            );
        }
    };
}

mozilla_site!(mozilla_bbc, "bbc-1", 200);
mozilla_site!(mozilla_nytimes, "nytimes-1", 200);
mozilla_site!(mozilla_medium, "medium-1", 100);
mozilla_site!(mozilla_wikipedia, "wikipedia", 500);
mozilla_site!(mozilla_cnn, "cnn", 100);
mozilla_site!(mozilla_guardian, "guardian-1", 200);
mozilla_site!(mozilla_wapo, "wapo-1", 200);
mozilla_site!(mozilla_ars, "ars-1", 200);
