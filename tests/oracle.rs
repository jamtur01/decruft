//! Oracle tests: compare decruft's extracted content against defuddle's
//! expected output using Jaccard sentence similarity.
//!
//! Each test asserts both a minimum word count (catches content loss)
//! and minimum Jaccard similarity (catches content drift).
//! Thresholds are set ~5% below actual measured values.

#![allow(clippy::panic, clippy::cast_precision_loss)]

use decruft::{DecruftOptions, parse};
use std::collections::HashSet;
use std::fs;

fn fixture_dir() -> String {
    format!("{}/tests/fixtures/defuddle", env!("CARGO_MANIFEST_DIR"))
}

fn expected_dir() -> String {
    format!("{}/tests/expected/defuddle", env!("CARGO_MANIFEST_DIR"))
}

fn load(dir: &str, name: &str, ext: &str) -> String {
    let path = format!("{dir}/{name}.{ext}");
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("fixture missing: {path}: {e}"))
}

fn url_from_html(html: &str) -> Option<String> {
    let start = html.find("<!-- {")?;
    let json_start = start + 5;
    let end = html[json_start..].find(" -->")?;
    let json_str = &html[json_start..json_start + end];
    let val: serde_json::Value = serde_json::from_str(json_str).ok()?;
    val.get("url").and_then(|v| v.as_str()).map(String::from)
}

fn expected_body(md: &str) -> &str {
    if let Some(first) = md.find("```") {
        if let Some(second) = md[first + 3..].find("```") {
            return md[first + 3 + second + 3..].trim();
        }
    }
    md.trim()
}

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

fn check(name: &str, min_sim: f64, min_words: usize) {
    let html = load(&fixture_dir(), name, "html");
    let expected_md = load(&expected_dir(), name, "md");
    let url = url_from_html(&html).unwrap_or_else(|| format!("https://example.com/{name}"));
    let mut opts = DecruftOptions::default();
    opts.url = Some(url);
    opts.markdown = true;
    let result = parse(&html, &opts);
    let words = result.content.split_whitespace().count();
    let sim = jaccard(
        &to_sentences(&result.content),
        &to_sentences(expected_body(&expected_md)),
    );
    assert!(
        words >= min_words,
        "{name}: {words} words < {min_words} minimum"
    );
    assert!(sim >= min_sim, "{name}: similarity {sim:.2} < {min_sim:.2}");
}

// ── Tier 1: high confidence ──────────────────────────────────────
// sim threshold ~5% below measured, word count ~20% below measured

#[test]
fn oracle_stephango() {
    check("general--stephango.com-buy-wisely", 0.66, 1000);
}
#[test]
fn oracle_daringfireball() {
    check(
        "general--daringfireball.net-2025-02-the_iphone_16e",
        0.88,
        1600,
    );
}
#[test]
fn oracle_trailing_related() {
    check("content-patterns--trailing-related-posts", 0.95, 50);
}
#[test]
fn oracle_card_grid() {
    check("content-patterns--card-grid-stripped-headings", 0.95, 150);
}
#[test]
fn oracle_maggieappleton() {
    check("footnotes--maggieappleton.com-xanadu-patterns", 0.63, 2000);
}
#[test]
fn oracle_dhammatalks() {
    check("issues--120-dhammatalks-footnotes", 0.78, 1600);
}
#[test]
fn oracle_header_wraps() {
    check("issues--header-wraps-content", 0.95, 200);
}
#[test]
fn oracle_lazy_image() {
    check("elements--lazy-image", 0.76, 250);
}
#[test]
fn oracle_mintlify() {
    check("codeblocks--mintlify", 0.45, 220);
}
#[test]
fn oracle_related_byline() {
    check("scoring--related-posts-byline", 0.80, 200);
}
#[test]
fn oracle_wikipedia() {
    check("general--wikipedia", 0.45, 680);
}
#[test]
fn oracle_obsidian_sync() {
    check(
        "general--obsidian.md-blog-verify-obsidian-sync-encryption",
        0.40,
        640,
    );
}
#[test]
fn oracle_code_boilerplate() {
    check(
        "content-patterns--code-block-boilerplate-and-trailing-section",
        0.83,
        450,
    );
}
#[test]
fn oracle_mdn() {
    check(
        "general--developer.mozilla.org-en-US-docs-Web-JavaScript-Reference-Global_Objects-Array",
        0.45,
        270,
    );
}

// ── Tier 2: format differences (htmd table padding, footnote format) ─

#[test]
fn oracle_scp_wiki() {
    check("general--scp-wiki.wikidot.com-scp-9935", 0.13, 190);
}
#[test]
fn oracle_cp4space() {
    check("general--cp4space-jordan-algebra", 0.14, 680);
}
#[test]
fn oracle_stripe() {
    check("codeblocks--stripe", 0.40, 150);
}
#[test]
fn oracle_complex_tables() {
    check("elements--complex-tables", 0.22, 160);
}
#[test]
fn oracle_partial_in_code() {
    check("issues--167-partial-selector-inside-code", 0.28, 75);
}
#[test]
fn oracle_table_with_links() {
    check("scoring--table-with-links", 0.28, 120);
}
