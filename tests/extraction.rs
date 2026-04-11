//! Content extraction correctness tests.
//!
//! Four levels:
//! 1. Upstream oracles — test against defuddle's and mozilla's expected outputs
//! 2. Fixture sweeps — all fixtures produce non-empty output
//! 3. Content assertions — "with/without" checks on hand-picked fixtures
//! 4. Edge cases
//!
//! This file does NOT test exact self-referential output (that's regression.rs)
//! or option toggles (that's behavior.rs).

#![allow(clippy::panic, clippy::cast_precision_loss)]

use decruft::{DecruftOptions, parse};
use std::collections::HashSet;
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
    serde_json::from_str::<serde_json::Value>(&html[json_start..json_start + end])
        .ok()?
        .get("url")
        .and_then(|v| v.as_str())
        .map(String::from)
}

/// Extract sentences from text (strips HTML/markdown) for overlap comparison.
fn to_sentences(text: &str) -> HashSet<String> {
    let stripped = text.replace("**", "").replace(['`', '#'], "");
    let stripped = regex_lite::Regex::new(r"<[^>]+>")
        .map(|re| re.replace_all(&stripped, " ").to_string())
        .unwrap_or(stripped);
    let stripped = regex_lite::Regex::new(r"\[([^\]]*)\]\([^)]*\)")
        .map(|re| re.replace_all(&stripped, "$1").to_string())
        .unwrap_or(stripped);
    let normalized = regex_lite::Regex::new(r"\s+")
        .map(|re| re.replace_all(&stripped, " ").to_string())
        .unwrap_or(stripped);
    normalized
        .split(['.', '!', '?'])
        .map(|s| s.trim().to_lowercase())
        .filter(|s| s.len() > 20)
        .collect()
}

fn jaccard(a: &HashSet<String>, b: &HashSet<String>) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    a.intersection(b).count() as f64 / a.union(b).count() as f64
}

fn str_field(val: &serde_json::Value, key: &str) -> String {
    val.get(key)
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

// ════════════════════════════════════════════════════════════════
// 1. Upstream oracles
// ════════════════════════════════════════════════════════════════

// ── Defuddle oracle: metadata + content from vendored .md files ──

/// Parse defuddle's expected .md file: JSON preamble → metadata, body → content.
fn parse_defuddle_expected(md: &str) -> Option<(serde_json::Value, String)> {
    let start = md.find("```json\n")?;
    let json_start = start + "```json\n".len();
    let json_end = md[json_start..].find("\n```")?;
    let json_str = &md[json_start..json_start + json_end];
    let meta: serde_json::Value = serde_json::from_str(json_str).ok()?;
    let body = md[json_start + json_end + 4..].trim().to_string();
    Some((meta, body))
}

#[test]
fn defuddle_oracle_metadata() {
    let fixture_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/defuddle");
    let expected_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/expected/defuddle");

    let mut failures = Vec::new();
    let mut total = 0;

    for entry in fs::read_dir(&fixture_dir).unwrap().flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("html") {
            continue;
        }
        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        let expected_path = expected_dir.join(format!("{name}.md"));
        let Ok(md) = fs::read_to_string(&expected_path) else {
            continue;
        };
        let Some((meta, _body)) = parse_defuddle_expected(&md) else {
            continue;
        };

        let html = fs::read_to_string(&path).unwrap();
        let url = url_from_html(&html).unwrap_or_else(|| format!("https://example.com/{name}"));
        let result = parse(&html, &opts(&url));

        total += 1;
        let expected_title = str_field(&meta, "title");
        let expected_author = str_field(&meta, "author");
        let expected_site = str_field(&meta, "site");
        let expected_published = str_field(&meta, "published");

        if !expected_title.is_empty() && result.title != expected_title {
            failures.push(format!(
                "{name}: title {:?} != {expected_title:?}",
                result.title
            ));
        }
        if !expected_author.is_empty() && result.author != expected_author {
            failures.push(format!(
                "{name}: author {:?} != {expected_author:?}",
                result.author
            ));
        }
        if !expected_site.is_empty() && result.site != expected_site {
            failures.push(format!(
                "{name}: site {:?} != {expected_site:?}",
                result.site
            ));
        }
        if !expected_published.is_empty() {
            let date_prefix = expected_published.split('T').next().unwrap_or("");
            if !date_prefix.is_empty() && !result.published.contains(date_prefix) {
                failures.push(format!(
                    "{name}: published {:?} missing {date_prefix:?}",
                    result.published
                ));
            }
        }
    }

    assert!(
        total >= 100,
        "expected ≥100 fixtures with metadata, got {total}"
    );
    // Track regressions. Current: ~69 mismatches (site/author field logic
    // differs from defuddle). Tighten as metadata extraction improves.
    assert!(
        failures.len() <= 70,
        "defuddle metadata regressed: {}/{total} mismatches:\n  {}",
        failures.len(),
        failures[..failures.len().min(10)].join("\n  ")
    );
}

#[test]
fn defuddle_oracle_content() {
    let fixture_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/defuddle");
    let expected_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/expected/defuddle");

    let mut low_sim = Vec::new();
    let mut total = 0;

    for entry in fs::read_dir(&fixture_dir).unwrap().flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("html") {
            continue;
        }
        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        let expected_path = expected_dir.join(format!("{name}.md"));
        let Ok(md) = fs::read_to_string(&expected_path) else {
            continue;
        };
        let Some((_meta, body)) = parse_defuddle_expected(&md) else {
            continue;
        };
        if body.is_empty() {
            continue;
        }

        let html = fs::read_to_string(&path).unwrap();
        let url = url_from_html(&html).unwrap_or_else(|| format!("https://example.com/{name}"));
        let mut md_opts = opts(&url);
        md_opts.markdown = true;
        let result = parse(&html, &md_opts);

        total += 1;
        let sim = jaccard(&to_sentences(&result.content), &to_sentences(&body));
        if sim < 0.10 && result.word_count > 20 {
            low_sim.push(format!("{name}: {sim:.2}"));
        }
    }

    assert!(
        total >= 100,
        "expected ≥100 fixtures with content, got {total}"
    );
    // Track regressions. Current: 14 below 0.10 (math, CJK, code blocks
    // where output format differs heavily from defuddle's markdown).
    assert!(
        low_sim.len() <= 14,
        "defuddle content regressed: {}/{total} below 0.10:\n  {}",
        low_sim.len(),
        low_sim.join("\n  ")
    );
}

// ── Mozilla oracle: metadata + content from expected-metadata.json / expected.html ──

#[test]
fn mozilla_oracle_metadata() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/mozilla");
    let mut failures = Vec::new();
    let mut total = 0;

    for entry in fs::read_dir(&dir).unwrap().flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        let meta_path = path.join("expected-metadata.json");
        let Ok(meta_str) = fs::read_to_string(&meta_path) else {
            continue;
        };
        let Ok(meta) = serde_json::from_str::<serde_json::Value>(&meta_str) else {
            continue;
        };
        let Ok(html) = fs::read_to_string(path.join("source.html")) else {
            continue;
        };

        total += 1;
        let result = parse(&html, &DecruftOptions::default());

        // Title
        let expected_title = str_field(&meta, "title");
        if !expected_title.is_empty()
            && result.title != expected_title
            && !result.title.contains(&expected_title)
            && !expected_title.contains(&result.title)
        {
            failures.push(format!(
                "{name}: title {:?} != {expected_title:?}",
                result.title
            ));
        }

        // Byline → author
        let expected_byline = str_field(&meta, "byline");
        if !expected_byline.is_empty()
            && result.author != expected_byline
            && !result.author.contains(&expected_byline)
            && !expected_byline.contains(&result.author)
        {
            failures.push(format!(
                "{name}: author {:?} != byline {expected_byline:?}",
                result.author
            ));
        }

        // Excerpt → description (skip — excerpt matching is too noisy
        // across implementations; title/byline/site/lang are the key fields)

        // Site name
        let expected_site = str_field(&meta, "siteName");
        if !expected_site.is_empty()
            && result.site != expected_site
            && !result.site.contains(&expected_site)
            && !expected_site.contains(&result.site)
        {
            failures.push(format!(
                "{name}: site {:?} != {expected_site:?}",
                result.site
            ));
        }

        // Language
        let expected_lang = str_field(&meta, "lang");
        if !expected_lang.is_empty() && result.language != expected_lang {
            failures.push(format!(
                "{name}: lang {:?} != {expected_lang:?}",
                result.language
            ));
        }
    }

    assert!(total >= 100, "expected ≥100 mozilla fixtures, got {total}");
    // Track regressions. Tighten as metadata parity improves.
    assert!(
        failures.len() <= 50,
        "mozilla metadata regressed: {}/{total} mismatches:\n  {}",
        failures.len(),
        failures.join("\n  ")
    );
}

#[test]
fn mozilla_oracle_content() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/mozilla");
    let mut low_sim = Vec::new();
    let mut total = 0;

    for entry in fs::read_dir(&dir).unwrap().flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        let expected_path = path.join("expected.html");
        let Ok(expected_html) = fs::read_to_string(&expected_path) else {
            continue;
        };
        let Ok(html) = fs::read_to_string(path.join("source.html")) else {
            continue;
        };

        let result = parse(&html, &DecruftOptions::default());
        if result.word_count < 20 {
            continue;
        }

        total += 1;
        let sim = jaccard(
            &to_sentences(&result.content),
            &to_sentences(&expected_html),
        );
        if sim < 0.15 {
            low_sim.push(format!("{name}: {sim:.2}"));
        }
    }

    assert!(
        total >= 100,
        "expected ≥100 mozilla fixtures with content, got {total}"
    );
    // Allow ≤8 low-similarity fixtures (tables, math, JS-dependent content)
    assert!(
        low_sim.len() <= 8,
        "mozilla content: {}/{total} below 0.15 similarity:\n  {}",
        low_sim.len(),
        low_sim.join("\n  ")
    );
}

// ════════════════════════════════════════════════════════════════
// 2. Fixture sweeps (non-empty extraction)
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
        let Ok(html) = fs::read_to_string(path.join("source.html")) else {
            continue;
        };
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
// 3. Content assertions (with/without)
// ════════════════════════════════════════════════════════════════

fn check_content(html: &str, url: &str, with: &[&str], without: &[&str]) {
    let result = parse(html, &opts(url));
    for phrase in with {
        assert!(
            result.content.contains(phrase),
            "missing {phrase:?} from {url}"
        );
    }
    for phrase in without {
        assert!(
            !result.content.contains(phrase),
            "found unwanted {phrase:?} from {url}"
        );
    }
}

#[test]
fn blog_content_and_clutter() {
    let html = load("complex_blog.html");
    check_content(
        &html,
        "https://example.com/article",
        &[
            "Rust's ownership system",
            "Borrowing and References",
            "String::from",
        ],
        &[
            "main-nav",
            "ad-banner",
            "Popular Posts",
            "cookie",
            "Privacy Policy",
            "share-twitter",
            "Comments (12)",
            "newsletter",
            "You Might Also Like",
        ],
    );
    let r = parse(&html, &opts("https://example.com/article"));
    assert_eq!(r.title, "Understanding Rust Ownership");
    assert_eq!(r.author, "Alice Chen");
    assert!(r.published.contains("2025-03-15"));
}

#[test]
fn news_content_and_clutter() {
    let html = load("news_article.html");
    check_content(
        &html,
        "https://example.com/article",
        &["Marine biologists", "Aurelia profundis", "<blockquote"],
        &[
            "inline-ad",
            "ADVERTISEMENT",
            "Trending Now",
            "BREAKING",
            "Enable notifications",
        ],
    );
    let r = parse(&html, &opts("https://example.com/article"));
    assert_eq!(r.title, "Scientists Discover New Species in Deep Ocean");
    assert_eq!(r.author, "Sarah Mitchell");
}

// ── Bug fix regressions ─────────────────────────────────────────

#[test]
fn stripe_code_blocks_preserved() {
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
    let html = load_defuddle("general--scp-wiki.wikidot.com-scp-9935.html");
    check_content(
        &html,
        "https://scp-wiki.wikidot.com/scp-9935",
        &["No relation to the Washington Nationals"],
        &[],
    );
}

#[test]
fn cp4space_title_and_bibliography() {
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
    let r = parse("", &DecruftOptions::default());
    assert!(r.content.is_empty() || r.word_count == 0);
}

#[test]
fn minimal_document() {
    let r = parse(
        "<html><body><p>Hello world</p></body></html>",
        &DecruftOptions::default(),
    );
    assert!(r.content.contains("Hello world"));
}

#[test]
fn semantic_html_preserved() {
    let html = r"<html><body><article>
        <h1>Title</h1>
        <p><strong>bold</strong> and <em>italic</em></p>
        <ul><li>Item</li></ul>
    </article></body></html>";
    let r = parse(html, &DecruftOptions::default());
    assert!(r.content.contains("<strong>bold</strong>"));
    assert!(r.content.contains("<em>italic</em>"));
}

// ── Mozilla individual sites ────────────────────────────────────

macro_rules! mozilla_site {
    ($name:ident, $fixture:expr, $min_words:expr) => {
        #[test]
        fn $name() {
            let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/mozilla")
                .join($fixture);
            let html = fs::read_to_string(dir.join("source.html"))
                .unwrap_or_else(|e| panic!("{}: {e}", $fixture));
            let r = parse(&html, &DecruftOptions::default());
            assert!(
                r.word_count >= $min_words,
                "{}: {} words < {}",
                $fixture,
                r.word_count,
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
