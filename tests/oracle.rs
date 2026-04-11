//! Oracle tests: compare decruft's extracted content against defuddle's
//! expected output using Jaccard sentence similarity.
//!
//! Thresholds are set ~5% below actual measured similarity. Any drop
//! below threshold indicates a real content regression.

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

fn check(name: &str, min_sim: f64) {
    let html = load(&fixture_dir(), name, "html");
    let expected_md = load(&expected_dir(), name, "md");
    let url = url_from_html(&html).unwrap_or_else(|| format!("https://example.com/{name}"));
    let mut opts = DecruftOptions::default();
    opts.url = Some(url);
    opts.markdown = true;
    let result = parse(&html, &opts);
    let sim = jaccard(
        &to_sentences(&result.content),
        &to_sentences(expected_body(&expected_md)),
    );
    assert!(sim >= min_sim, "{name}: similarity {sim:.2} < {min_sim:.2}");
}

// ── Tier 1: thresholds set ~5% below measured actual ─────────────
// Measured 2026-04-09. Actual values in comments.

#[test]
fn oracle_stephango() {
    check("general--stephango.com-buy-wisely", 0.66);
} // actual: 0.717
#[test]
fn oracle_daringfireball() {
    check("general--daringfireball.net-2025-02-the_iphone_16e", 0.88);
} // actual: 0.926
#[test]
fn oracle_trailing_related() {
    check("content-patterns--trailing-related-posts", 0.95);
} // actual: 1.000
#[test]
fn oracle_card_grid() {
    check("content-patterns--card-grid-stripped-headings", 0.95);
} // actual: 1.000
#[test]
fn oracle_maggieappleton() {
    check("footnotes--maggieappleton.com-xanadu-patterns", 0.63);
} // actual: 0.681
#[test]
fn oracle_dhammatalks() {
    check("issues--120-dhammatalks-footnotes", 0.78);
} // actual: 0.833
#[test]
fn oracle_header_wraps() {
    check("issues--header-wraps-content", 0.95);
} // actual: 1.000
#[test]
fn oracle_lazy_image() {
    check("elements--lazy-image", 0.76);
} // actual: 0.808
#[test]
fn oracle_mintlify() {
    check("codeblocks--mintlify", 0.45);
} // actual: 0.500
#[test]
fn oracle_related_byline() {
    check("scoring--related-posts-byline", 0.80);
} // actual: 0.846
#[test]
fn oracle_wikipedia() {
    check("general--wikipedia", 0.84);
} // actual: 0.895
#[test]
fn oracle_obsidian_sync() {
    check(
        "general--obsidian.md-blog-verify-obsidian-sync-encryption",
        0.40,
    );
} // actual: 0.457
#[test]
fn oracle_code_boilerplate() {
    check(
        "content-patterns--code-block-boilerplate-and-trailing-section",
        0.83,
    );
} // actual: 0.879
#[test]
fn oracle_mdn() {
    check(
        "general--developer.mozilla.org-en-US-docs-Web-JavaScript-Reference-Global_Objects-Array",
        0.45,
    );
} // actual: 0.500

// ── Tier 2: known bugs — thresholds at current level ─────────────

#[test]
fn oracle_scp_wiki() {
    check("general--scp-wiki.wikidot.com-scp-9935", 0.15);
} // #7  actual: 0.182
#[test]
fn oracle_cp4space() {
    check("general--cp4space-jordan-algebra", 0.12);
} // math actual: 0.154
#[test]
fn oracle_stripe() {
    check("codeblocks--stripe", 0.22);
} // #8  actual: 0.263
#[test]
fn oracle_complex_tables() {
    check("elements--complex-tables", 0.22);
} // #6  actual: 0.250
#[test]
fn oracle_partial_in_code() {
    check("issues--167-partial-selector-inside-code", 0.28);
} //     actual: 0.333
#[test]
fn oracle_table_with_links() {
    check("scoring--table-with-links", 0.28);
} //     actual: 0.333
