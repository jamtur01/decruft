//! Content extraction correctness tests.
//!
//! Every fixture must:
//! 1. Extract non-empty content
//! 2. Match its metadata JSON exactly (title, author, site, published, language)
//!
//! The metadata JSON files in tests/expected/metadata/ ARE the oracle.
//! When extraction improves, regenerate with:
//!     `cargo test --test extraction -- --ignored regenerate_metadata`

#![allow(clippy::panic)]

use decruft::{DecruftOptions, parse};
use std::fs;
use std::path::PathBuf;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn metadata_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/expected/metadata")
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

// ════════════════════════════════════════════════════════════════
// 1. Non-empty extraction sweep
// ════════════════════════════════════════════════════════════════

#[test]
fn all_fixtures_extract_content() {
    let mut failures = Vec::new();
    let mut total = 0;

    for entry in fs::read_dir(fixtures_dir()).unwrap().flatten() {
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

    assert!(
        failures.is_empty(),
        "empty extractions ({}/{}): {failures:?}",
        failures.len(),
        total
    );
}

// ════════════════════════════════════════════════════════════════
// 2. Metadata exact match
// ════════════════════════════════════════════════════════════════

#[test]
fn all_fixtures_match_metadata() {
    let meta_dir = metadata_dir();
    let mut failures = Vec::new();
    let mut total = 0;

    let mut missing_meta = Vec::new();

    for entry in fs::read_dir(fixtures_dir()).unwrap().flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("html") {
            continue;
        }
        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        let meta_path = meta_dir.join(format!("{name}.json"));

        let Ok(meta_str) = fs::read_to_string(&meta_path) else {
            missing_meta.push(name);
            continue;
        };
        let Ok(expected) = serde_json::from_str::<serde_json::Value>(&meta_str) else {
            missing_meta.push(name);
            continue;
        };

        let html = fs::read_to_string(&path).unwrap();
        let url = url_from_html(&html).unwrap_or_else(|| format!("https://example.com/{name}"));
        let result = parse(&html, &opts(&url));

        total += 1;

        let exp_title = expected["title"].as_str().unwrap_or("");
        let exp_author = expected["author"].as_str().unwrap_or("");
        let exp_site = expected["site"].as_str().unwrap_or("");
        let exp_published = expected["published"].as_str().unwrap_or("");
        let exp_lang = expected["language"].as_str().unwrap_or("");

        let mut mismatches = Vec::new();
        if result.title != exp_title {
            mismatches.push("title");
        }
        if result.author != exp_author {
            mismatches.push("author");
        }
        if result.site != exp_site {
            mismatches.push("site");
        }
        if result.published != exp_published {
            mismatches.push("published");
        }
        if !exp_lang.is_empty() && result.language != exp_lang {
            mismatches.push("language");
        }

        if !mismatches.is_empty() {
            failures.push(format!("{name}: {}", mismatches.join(", ")));
        }
    }

    assert!(
        missing_meta.is_empty(),
        "fixtures missing metadata JSON:\n  {}",
        missing_meta.join("\n  ")
    );
    assert!(
        failures.is_empty(),
        "{}/{total} metadata mismatches:\n  {}",
        failures.len(),
        failures[..failures.len().min(20)].join("\n  ")
    );
}

// ════════════════════════════════════════════════════════════════
// 3. Content assertions (with/without)
// ════════════════════════════════════════════════════════════════

fn load(name: &str) -> String {
    let path = fixtures_dir().join(name);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn check(html: &str, url: &str, with: &[&str], without: &[&str]) {
    let result = parse(html, &opts(url));
    for p in with {
        assert!(result.content.contains(p), "missing {p:?} from {url}");
    }
    for p in without {
        assert!(!result.content.contains(p), "unwanted {p:?} from {url}");
    }
}

#[test]
fn blog_content_and_clutter() {
    let html = load("complex_blog.html");
    check(
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
}

#[test]
fn news_content_and_clutter() {
    let html = load("news_article.html");
    check(
        &html,
        "https://example.com/article",
        &["Marine biologists", "Aurelia profundis", "<blockquote"],
        &["inline-ad", "ADVERTISEMENT", "Trending Now", "BREAKING"],
    );
}

#[test]
fn stripe_code_blocks_preserved() {
    let html = load("codeblocks--stripe.html");
    check(
        &html,
        "https://stripe.com/docs",
        &["paymentMiddleware", "curl"],
        &[],
    );
}

#[test]
fn scp_wiki_footnotes_preserved() {
    let html = load("general--scp-wiki.wikidot.com-scp-9935.html");
    check(
        &html,
        "https://scp-wiki.wikidot.com/scp-9935",
        &["No relation to the Washington Nationals"],
        &[],
    );
}

#[test]
fn cp4space_title_and_bibliography() {
    let html = load("general--cp4space-jordan-algebra.html");
    check(
        &html,
        "https://cp4space.hatsya.com/2020/10/28/the-exceptional-jordan-algebra/",
        &["exceptional Jordan algebra", "John Baez"],
        &[],
    );
}

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

// ════════════════════════════════════════════════════════════════
// Metadata regeneration (run with --ignored)
// ════════════════════════════════════════════════════════════════

#[test]
#[ignore = "run manually to regenerate metadata expectations"]
fn regenerate_metadata() {
    let meta_dir = metadata_dir();
    fs::create_dir_all(&meta_dir).unwrap();

    let mut count = 0;
    for entry in fs::read_dir(fixtures_dir()).unwrap().flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("html") {
            continue;
        }
        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        let html = fs::read_to_string(&path).unwrap();
        let url = url_from_html(&html).unwrap_or_else(|| format!("https://example.com/{name}"));
        let result = parse(&html, &opts(&url));

        let meta = serde_json::json!({
            "title": result.title,
            "author": result.author,
            "site": result.site,
            "published": result.published,
            "language": result.language,
        });
        fs::write(
            meta_dir.join(format!("{name}.json")),
            serde_json::to_string_pretty(&meta).unwrap(),
        )
        .unwrap();
        count += 1;
    }

    panic!("Regenerated {count} metadata files. Review: git diff tests/expected/metadata/");
}
