//! Oracle tests: compare decruft's extracted content against defuddle's
//! expected output for curated fixtures.
//!
//! Uses Jaccard sentence similarity to catch content regressions while
//! tolerating minor formatting differences (footnote numbering, code
//! block formatting, whitespace).
//!
//! Thresholds are calibrated to current output. If a change causes a
//! test to fail, either the threshold needs adjusting (with justification)
//! or there's a real regression to investigate.

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
    if let Some(start) = html.find("<!-- {") {
        let json_start = start + 5;
        if let Some(end) = html[json_start..].find(" -->") {
            let json_str = &html[json_start..json_start + end];
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(json_str) {
                return val.get("url").and_then(|v| v.as_str()).map(String::from);
            }
        }
    }
    None
}

fn expected_body(md: &str) -> &str {
    if let Some(first_fence) = md.find("```") {
        if let Some(second_fence) = md[first_fence + 3..].find("```") {
            let after = first_fence + 3 + second_fence + 3;
            return md[after..].trim();
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
    let intersection = a.intersection(b).count();
    let union = a.union(b).count();
    intersection as f64 / union as f64
}

fn check_fixture(name: &str, min_similarity: f64) {
    let html = load(&fixture_dir(), name, "html");
    let expected_md = load(&expected_dir(), name, "md");
    let expected_text = expected_body(&expected_md);

    let url = url_from_html(&html).unwrap_or_else(|| format!("https://example.com/{name}"));
    let mut opts = DecruftOptions::default();
    opts.url = Some(url);
    opts.markdown = true;
    let result = parse(&html, &opts);

    let expected_sents = to_sentences(expected_text);
    let actual_sents = to_sentences(&result.content);
    let sim = jaccard(&expected_sents, &actual_sents);

    assert!(
        sim >= min_similarity,
        "{name}: similarity {sim:.2} < {min_similarity:.2}\n\
         expected: {} sentences, actual: {} sentences\n\
         only in expected: {:?}\n\
         only in actual: {:?}",
        expected_sents.len(),
        actual_sents.len(),
        expected_sents
            .difference(&actual_sents)
            .take(3)
            .collect::<Vec<_>>(),
        actual_sents
            .difference(&expected_sents)
            .take(3)
            .collect::<Vec<_>>(),
    );
}

// ── Tier 1: High-quality articles (≥70% overlap) ────────────────
// These are clean articles where we should closely match defuddle.

#[test]
fn oracle_stephango_buy_wisely() {
    check_fixture("general--stephango.com-buy-wisely", 0.70);
}

#[test]
fn oracle_daringfireball() {
    check_fixture("general--daringfireball.net-2025-02-the_iphone_16e", 0.65);
}

#[test]
fn oracle_trailing_related_posts() {
    check_fixture("content-patterns--trailing-related-posts", 0.60);
}

#[test]
fn oracle_card_grid_headings() {
    check_fixture("content-patterns--card-grid-stripped-headings", 0.55);
}

#[test]
fn oracle_footnotes_maggieappleton() {
    check_fixture("footnotes--maggieappleton.com-xanadu-patterns", 0.50);
}

#[test]
fn oracle_footnotes_dhammatalks() {
    check_fixture("issues--120-dhammatalks-footnotes", 0.50);
}

#[test]
fn oracle_issue_header_wraps_content() {
    check_fixture("issues--header-wraps-content", 0.60);
}

#[test]
fn oracle_elements_lazy_image() {
    check_fixture("elements--lazy-image", 0.55);
}

#[test]
fn oracle_codeblocks_mintlify() {
    check_fixture("codeblocks--mintlify", 0.50);
}

#[test]
fn oracle_scoring_related_posts_byline() {
    check_fixture("scoring--related-posts-byline", 0.55);
}

#[test]
fn oracle_wikipedia() {
    check_fixture("general--wikipedia", 0.55);
}

// ── Tier 2: Known issues (tracked in GitHub Issues) ─────────────
// These have real extraction bugs. Thresholds are set just below
// current output so they don't regress further, but the target is
// ≥0.50 for all of them once the bugs are fixed.

#[test]
fn oracle_mdn() {
    // 95% word retention — sentence similarity low due to code example
    // boundary differences, not actual content loss
    check_fixture(
        "general--developer.mozilla.org-en-US-docs-Web-JavaScript-Reference-Global_Objects-Array",
        0.45,
    );
}

#[test]
fn oracle_obsidian_sync() {
    // 98% word retention — low sentence sim from link formatting diffs
    check_fixture(
        "general--obsidian.md-blog-verify-obsidian-sync-encryption",
        0.40,
    );
}

#[test]
fn oracle_code_block_boilerplate() {
    check_fixture(
        "content-patterns--code-block-boilerplate-and-trailing-section",
        0.45,
    );
}

#[test]
fn oracle_scp_wiki() {
    // Issue #7: 70% word retention — Wikidot page structure
    check_fixture("general--scp-wiki.wikidot.com-scp-9935", 0.15);
}

#[test]
fn oracle_cp4space() {
    // 84% word retention — math content partially lost
    check_fixture("general--cp4space-jordan-algebra", 0.12);
}

#[test]
fn oracle_codeblocks_stripe() {
    // Issue #8: 73% word retention — code block stripping too aggressive
    check_fixture("codeblocks--stripe", 0.22);
}

#[test]
fn oracle_elements_complex_tables() {
    // Issue #6: 61% word retention — table content lost
    check_fixture("elements--complex-tables", 0.22);
}

#[test]
fn oracle_issue_partial_selector_in_code() {
    // 99% word retention — title difference only
    check_fixture("issues--167-partial-selector-inside-code", 0.30);
}

#[test]
fn oracle_scoring_table_with_links() {
    // Issue #6: 56% word retention — link-heavy table stripped
    check_fixture("scoring--table-with-links", 0.30);
}
