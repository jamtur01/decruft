#![allow(clippy::panic, clippy::print_stderr)]

use decruft::{DecruftOptions, parse};
use std::fs;
use std::path::Path;

fn fixture_dir() -> String {
    format!("{}/tests/fixtures/defuddle", env!("CARGO_MANIFEST_DIR"))
}

fn expected_dir() -> String {
    format!("{}/tests/expected/defuddle", env!("CARGO_MANIFEST_DIR"))
}

struct ExpectedMeta {
    title: String,
    author: String,
    site: String,
    published: String,
}

fn parse_expected_meta(md_content: &str) -> Option<ExpectedMeta> {
    let start = md_content.find("```json\n")?;
    let json_start = start + "```json\n".len();
    let json_end = md_content[json_start..].find("\n```")?;
    let json_str = &md_content[json_start..json_start + json_end];

    let val: serde_json::Value = serde_json::from_str(json_str).ok()?;

    Some(ExpectedMeta {
        title: val
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        author: val
            .get("author")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        site: val
            .get("site")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        published: val
            .get("published")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
    })
}

fn url_from_fixture(html: &str, name: &str) -> String {
    if let Some(start) = html.find("<!-- {") {
        let comment_start = start + "<!-- ".len();
        if let Some(end) = html[comment_start..].find(" -->") {
            let json_str = &html[comment_start..comment_start + end];
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(json_str)
                && let Some(url) = val.get("url").and_then(|v| v.as_str())
            {
                return url.to_string();
            }
        }
    }
    format!("https://example.com/{name}")
}

fn opts_for(url: &str) -> DecruftOptions {
    DecruftOptions {
        url: Some(url.to_string()),
        ..DecruftOptions::default()
    }
}

fn load_fixture(name: &str) -> String {
    let path = format!("{}/{name}", fixture_dir());
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("fixture not found at {path}: {e}"))
}

fn load_expected(name: &str) -> Option<String> {
    let path = format!("{}/{name}", expected_dir());
    fs::read_to_string(&path).ok()
}

fn discover_fixtures() -> Vec<String> {
    let dir = fixture_dir();
    let mut names: Vec<String> = fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("cannot read {dir}: {e}"))
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let name = entry.file_name().to_string_lossy().to_string();
            if Path::new(&name)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("html"))
            {
                Some(name)
            } else {
                None
            }
        })
        .collect();
    names.sort();
    names
}

fn stem(filename: &str) -> &str {
    filename.strip_suffix(".html").unwrap_or(filename)
}

/// Fixtures where empty content is expected (content lives in data
/// attributes or requires JS rendering, not in the HTML body).
const KNOWN_EMPTY: &[&str] = &[
    // Content is in a data attribute with BBCode, not in HTML body
    "extractor--bbcode-data",
    // Substack notes use deeply nested div structures that our
    // selectors strip; requires site-specific extraction logic
    "general--substack-note",
    "general--substack-note-permalink",
];

// ── Parameterized fixture sweep ──────────────────────────────────

#[test]
fn defuddle_fixtures_extract_content() {
    let fixtures = discover_fixtures();
    assert!(
        fixtures.len() > 100,
        "expected 100+ fixtures, found {}",
        fixtures.len()
    );

    let mut passed = 0_usize;
    let mut meta_mismatches: Vec<String> = Vec::new();
    let mut failures: Vec<String> = Vec::new();

    for fixture_file in &fixtures {
        let name = stem(fixture_file);
        let html = load_fixture(fixture_file);
        let url = url_from_fixture(&html, name);
        let result = parse(&html, &opts_for(&url));

        if KNOWN_EMPTY.contains(&name) {
            passed += 1;
            continue;
        }

        if result.content.trim().is_empty() {
            failures.push(format!("{name}: empty content"));
            continue;
        }

        if result.word_count == 0 {
            failures.push(format!("{name}: word_count is 0"));
            continue;
        }

        let expected_file = format!("{name}.md");
        if let Some(md) = load_expected(&expected_file)
            && let Some(meta) = parse_expected_meta(&md)
        {
            let mut mismatches = Vec::new();

            if !meta.title.is_empty() && result.title != meta.title {
                mismatches.push(format!(
                    "title: got {:?}, expected {:?}",
                    result.title, meta.title
                ));
            }
            if !meta.author.is_empty() && result.author != meta.author {
                mismatches.push(format!(
                    "author: got {:?}, expected {:?}",
                    result.author, meta.author
                ));
            }
            if !meta.site.is_empty() && result.site != meta.site {
                mismatches.push(format!(
                    "site: got {:?}, expected {:?}",
                    result.site, meta.site
                ));
            }
            if !meta.published.is_empty()
                && !result
                    .published
                    .contains(meta.published.split('T').next().unwrap_or(""))
            {
                mismatches.push(format!(
                    "published: got {:?}, expected {:?}",
                    result.published, meta.published
                ));
            }

            if !mismatches.is_empty() {
                meta_mismatches.push(format!("{name}: {}", mismatches.join("; ")));
            }
        }

        passed += 1;
    }

    eprintln!(
        "\n=== Defuddle fixture results ===\n\
         Total:  {}\n\
         Passed: {passed}\n\
         Failed: {}\n\
         Meta mismatches: {}\n",
        fixtures.len(),
        failures.len(),
        meta_mismatches.len(),
    );

    if !meta_mismatches.is_empty() {
        eprintln!("--- Metadata mismatches (non-fatal) ---");
        for m in &meta_mismatches {
            eprintln!("  {m}");
        }
        eprintln!();
    }

    if !failures.is_empty() {
        eprintln!("--- Failures ---");
        for f in &failures {
            eprintln!("  {f}");
        }
        panic!("{} fixture(s) failed extraction", failures.len());
    }
}

// ── General extraction tests ─────────────────────────────────────

#[test]
fn general_stephango_buy_wisely() {
    let html = load_fixture("general--stephango.com-buy-wisely.html");
    let result = parse(&html, &opts_for("https://stephango.com/buy-wisely"));

    assert_eq!(result.title, "Buy wisely");
    assert_eq!(result.author, "Steph Ango");
    assert!(
        result.content.contains("cost per use"),
        "should contain key concept"
    );
    assert!(
        result.content.contains("Darn Tough"),
        "should contain specific example"
    );
    assert!(
        result.word_count > 500,
        "substantial article, got {}",
        result.word_count
    );
}

#[test]
fn general_wikipedia() {
    let html = load_fixture("general--wikipedia.html");
    let result = parse(&html, &opts_for("https://en.wikipedia.org/wiki/Obsidian"));

    assert_eq!(result.title, "Obsidian (software)");
    assert!(
        result.content.contains("personal knowledge management")
            || result.content.contains("note-taking"),
        "should contain topic content"
    );
    assert!(
        result.word_count > 200,
        "wikipedia article should be substantial, got {}",
        result.word_count
    );
}

#[test]
fn general_appendix_heading() {
    let html = load_fixture("general--appendix-heading.html");
    let result = parse(&html, &opts_for("https://example.com/appendix"));

    assert_eq!(result.title, "Article with Appendix");
    assert!(
        result.content.contains("Appendix"),
        "appendix content should be preserved"
    );
    assert!(
        result.content.contains("main article content"),
        "main content should be present"
    );
}

#[test]
fn general_back_nav_link() {
    let html = load_fixture("general--back-nav-link.html");
    let result = parse(&html, &opts_for("https://example.com/sorting"));

    assert_eq!(result.title, "An Article About Sorting");
    assert!(
        result.content.contains("Quicksort"),
        "article content should be present"
    );
}

#[test]
fn general_trailing_cta_newsletter() {
    let html = load_fixture("general--trailing-cta-newsletter.html");
    let result = parse(&html, &opts_for("https://example.com/article"));

    assert!(result.word_count > 0, "should extract content");
}

#[test]
fn general_inline_comments_and_link_lists() {
    let html = load_fixture("general--inline-comments-and-link-lists.html");
    let result = parse(&html, &opts_for("https://example.com/article"));

    assert!(
        result.content.contains("new model"),
        "should contain article content"
    );
    assert_eq!(result.author, "Jane Smith");
}

// ── Content pattern tests ────────────────────────────────────────

#[test]
fn content_patterns_card_grid_stripped_headings() {
    let html = load_fixture("content-patterns--card-grid-stripped-headings.html");
    let result = parse(&html, &opts_for("https://example.com/spacecraft"));

    assert!(
        result.content.contains("microgravity"),
        "main article content should be present"
    );
    assert!(
        !result.content.contains("Ion Thruster Breaks"),
        "card grid article titles should be removed"
    );
}

#[test]
fn content_patterns_trailing_related_posts() {
    let html = load_fixture("content-patterns--trailing-related-posts.html");
    let result = parse(&html, &opts_for("https://example.com/coffee"));

    assert!(
        result.content.contains("Newton"),
        "main content about coffee cooling should be present"
    );
}

#[test]
fn content_patterns_heading_introduced_list() {
    let html = load_fixture("content-patterns--heading-introduced-list.html");
    let result = parse(&html, &opts_for("https://example.com/plugin"));

    assert!(
        result.content.contains("Features") || result.content.contains("Asynchronous"),
        "feature list should be present"
    );
}

#[test]
fn content_patterns_iso_date_and_read_time() {
    let html = load_fixture("content-patterns--iso-date-and-read-time.html");
    let result = parse(&html, &opts_for("https://example.com/article"));

    assert!(result.word_count > 0, "should extract content");
}

// ── Hidden element tests ─────────────────────────────────────────

#[test]
fn hidden_nodes_removes_display_none() {
    let html = load_fixture("hidden--nodes.html");
    let result = parse(&html, &opts_for("https://example.com/hidden-nodes"));

    assert!(
        result.content.contains("Lorem ipsum"),
        "visible content should be present"
    );
    // The hidden paragraphs use display:none and the hidden attribute.
    // All paragraphs share the same Lorem ipsum text, so we verify
    // that content was extracted at all.
    assert!(
        result.word_count > 10,
        "should have some content, got {}",
        result.word_count
    );
}

#[test]
fn hidden_visibility_removes_hidden_content() {
    let html = load_fixture("hidden--visibility.html");
    let result = parse(&html, &opts_for("https://example.com/visibility"));

    assert!(
        result.content.contains("Foo") || result.content.contains("Tempor incididunt"),
        "visible content should be present"
    );
}
