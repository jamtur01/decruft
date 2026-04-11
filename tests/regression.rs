//! Golden file regression tests — the primary gate.
//!
//! Each fixture has golden expected outputs (HTML and markdown).
//! Tests compare current extraction byte-for-byte against the golden
//! file. Any difference means extraction behavior changed.
//!
//! Fixture URLs are read from HTML comments (`<!-- {"url":"..."} -->`)
//! so URL-dependent behavior (extractor routing, relative URL resolution)
//! is exercised.
//!
//! To regenerate after intentional changes:
//!
//!     cargo test --test regression -- --ignored regenerate
//!
//! Then review with `git diff tests/expected/`.

#![allow(clippy::panic)]

use decruft::{DecruftOptions, parse};
use std::fs;
use std::path::PathBuf;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn golden_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/expected/golden")
}

fn golden_md_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/expected/golden-markdown")
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

fn opts_for(html: &str, name: &str) -> DecruftOptions {
    let mut o = DecruftOptions::default();
    o.url = Some(url_from_html(html).unwrap_or_else(|| format!("https://example.com/{name}")));
    o
}

#[test]
fn golden_html() {
    let golden = golden_dir();
    let mut failures = Vec::new();
    let mut missing = Vec::new();

    for entry in fs::read_dir(fixtures_dir()).unwrap().flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("html") {
            continue;
        }
        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        let golden_path = golden.join(format!("{name}.html"));

        let Ok(expected) = fs::read_to_string(&golden_path) else {
            missing.push(name);
            continue;
        };

        let html = fs::read_to_string(&path).unwrap();
        let result = parse(&html, &opts_for(&html, &name));

        if result.content != expected {
            failures.push(name);
        }
    }

    assert!(
        missing.is_empty(),
        "fixtures missing golden HTML:\n  {}",
        missing.join("\n  ")
    );
    assert!(
        failures.is_empty(),
        "{} golden HTML mismatches:\n  {}\n\n\
         Regenerate: cargo test --test regression -- --ignored regenerate",
        failures.len(),
        failures.join("\n  ")
    );
}

#[test]
fn golden_markdown() {
    let golden = golden_md_dir();
    let mut failures = Vec::new();
    let mut missing = Vec::new();

    for entry in fs::read_dir(fixtures_dir()).unwrap().flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("html") {
            continue;
        }
        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        let golden_path = golden.join(format!("{name}.md"));

        let Ok(expected) = fs::read_to_string(&golden_path) else {
            missing.push(name);
            continue;
        };

        let html = fs::read_to_string(&path).unwrap();
        let mut opts = opts_for(&html, &name);
        opts.markdown = true;
        let result = parse(&html, &opts);

        if result.content != expected {
            failures.push(name);
        }
    }

    assert!(
        missing.is_empty(),
        "fixtures missing golden markdown:\n  {}",
        missing.join("\n  ")
    );
    assert!(
        failures.is_empty(),
        "{} golden markdown mismatches:\n  {}\n\n\
         Regenerate: cargo test --test regression -- --ignored regenerate",
        failures.len(),
        failures.join("\n  ")
    );
}

#[test]
#[ignore = "run manually to regenerate golden files"]
fn regenerate() {
    let fixtures = fixtures_dir();
    let golden = golden_dir();
    let golden_md = golden_md_dir();
    fs::create_dir_all(&golden).unwrap();
    fs::create_dir_all(&golden_md).unwrap();

    let mut count = 0;
    for entry in fs::read_dir(&fixtures).unwrap().flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("html") {
            continue;
        }
        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        let html = fs::read_to_string(&path).unwrap();
        let base_opts = opts_for(&html, &name);

        let result = parse(&html, &base_opts);
        fs::write(golden.join(format!("{name}.html")), &result.content).unwrap();

        let mut md_opts = base_opts;
        md_opts.markdown = true;
        let md_result = parse(&html, &md_opts);
        fs::write(golden_md.join(format!("{name}.md")), &md_result.content).unwrap();

        count += 1;
    }

    panic!("Regenerated {count} golden files (HTML + markdown). Review: git diff tests/expected/");
}
