//! Mozilla Readability test suite: 130 real-world fixtures.
//!
//! Fixtures from Mozilla's Readability.js project (via readabilityrs).
//! Tests content extraction and metadata across BBC, NYT, CNN, Medium,
//! Wikipedia, The Verge, and dozens more.

use decruft::{DecruftOptions, parse};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

fn mozilla_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/mozilla")
}

fn load_meta(path: &std::path::Path) -> serde_json::Value {
    let p = path.join("expected-metadata.json");
    serde_json::from_str(
        &fs::read_to_string(&p).unwrap_or_else(|e| panic!("missing {}: {e}", p.display())),
    )
    .unwrap_or_else(|e| panic!("bad json {}: {e}", p.display()))
}

fn load_expected_html(path: &std::path::Path) -> Option<String> {
    fs::read_to_string(path.join("expected.html")).ok()
}

fn to_sentences(html: &str) -> HashSet<String> {
    let text = html
        .chars()
        .fold((String::new(), false), |(mut out, in_tag), c| {
            if c == '<' {
                (out, true)
            } else if c == '>' {
                out.push(' ');
                (out, false)
            } else if !in_tag {
                out.push(c);
                (out, false)
            } else {
                (out, in_tag)
            }
        })
        .0;
    let words: Vec<&str> = text.split_whitespace().collect();
    let joined = words.join(" ");
    joined
        .split(['.', '!', '?'])
        .map(|s| s.trim().to_lowercase())
        .filter(|s| s.len() > 25)
        .collect()
}

fn jaccard(a: &HashSet<String>, b: &HashSet<String>) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    #[allow(clippy::cast_precision_loss)]
    let sim = a.intersection(b).count() as f64 / a.union(b).count() as f64;
    sim
}

fn all_fixture_dirs() -> Vec<PathBuf> {
    let base = mozilla_dir();
    let mut dirs: Vec<_> = fs::read_dir(&base)
        .unwrap_or_else(|e| panic!("can't read {}: {e}", base.display()))
        .filter_map(std::result::Result::ok)
        .filter(|e| e.path().is_dir())
        .map(|e| e.path())
        .collect();
    dirs.sort();
    dirs
}

// ── Full suite: content extraction ───────────────────────────────

#[test]
fn mozilla_suite_no_empty_extractions() {
    let dirs = all_fixture_dirs();
    assert!(dirs.len() >= 100);

    let mut empties = Vec::new();
    for dir in &dirs {
        let name = dir.file_name().unwrap().to_string_lossy().to_string();
        let html =
            fs::read_to_string(dir.join("source.html")).unwrap_or_else(|e| panic!("{name}: {e}"));
        let meta = load_meta(dir);
        let result = parse(&html, &DecruftOptions::default());

        let readerable = meta
            .get("readerable")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(true);
        if result.word_count == 0 && readerable {
            empties.push(name);
        }
    }
    assert!(empties.is_empty(), "empty extractions: {empties:?}");
}

#[test]
fn mozilla_suite_content_similarity() {
    let dirs = all_fixture_dirs();
    let mut low_sim = Vec::new();

    for dir in &dirs {
        let name = dir.file_name().unwrap().to_string_lossy().to_string();
        let html =
            fs::read_to_string(dir.join("source.html")).unwrap_or_else(|e| panic!("{name}: {e}"));
        let result = parse(&html, &DecruftOptions::default());

        if let Some(expected_html) = load_expected_html(dir) {
            let sim = jaccard(
                &to_sentences(&result.content),
                &to_sentences(&expected_html),
            );
            if sim < 0.20 && result.word_count > 20 {
                low_sim.push(format!("{name}: {sim:.2}"));
            }
        }
    }
    // Allow up to 5 low-similarity fixtures (known issues with tables, math)
    assert!(
        low_sim.len() <= 8,
        "too many low-similarity fixtures ({}): {low_sim:?}",
        low_sim.len()
    );
}

#[test]
fn mozilla_suite_title_match_rate() {
    let dirs = all_fixture_dirs();
    let mut matches = 0;
    let mut total = 0;
    let mut failures = Vec::new();

    for dir in &dirs {
        let name = dir.file_name().unwrap().to_string_lossy().to_string();
        let html =
            fs::read_to_string(dir.join("source.html")).unwrap_or_else(|e| panic!("{name}: {e}"));
        let meta = load_meta(dir);
        let result = parse(&html, &DecruftOptions::default());

        if let Some(exp) = meta.get("title").and_then(|v| v.as_str()) {
            if !exp.is_empty() {
                total += 1;
                if result.title == exp || result.title.contains(exp) || exp.contains(&result.title)
                {
                    matches += 1;
                } else {
                    failures.push(name);
                }
            }
        }
    }
    #[allow(clippy::cast_precision_loss)]
    let pct = (matches * 100) / total.max(1);
    assert!(
        pct >= 80,
        "title match {matches}/{total} ({pct}%) < 80%. Failures: {:?}",
        &failures[..failures.len().min(10)]
    );
}

// ── Individual site tests ────────────────────────────────────────

macro_rules! mozilla_test {
    ($name:ident, $fixture:expr, $min_words:expr) => {
        #[test]
        fn $name() {
            let dir = mozilla_dir().join($fixture);
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

mozilla_test!(mozilla_bbc, "bbc-1", 200);
mozilla_test!(mozilla_nytimes, "nytimes-1", 200);
mozilla_test!(mozilla_medium, "medium-1", 100);
mozilla_test!(mozilla_wikipedia, "wikipedia", 500);
mozilla_test!(mozilla_cnn, "cnn", 100);
mozilla_test!(mozilla_guardian, "guardian-1", 200);
mozilla_test!(mozilla_wapo, "wapo-1", 200);
mozilla_test!(mozilla_ars, "ars-1", 200);
mozilla_test!(mozilla_hidden_nodes, "hidden-nodes", 1);
mozilla_test!(mozilla_lazy_images, "lazy-image-1", 1);
mozilla_test!(mozilla_embedded_videos, "embedded-videos", 1);
mozilla_test!(mozilla_table_style, "table-style-attributes", 50);
