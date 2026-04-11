//! Golden file regression tests — the primary gate.
//!
//! Each fixture has a golden expected output in tests/expected/golden/.
//! Tests compare current extraction against the golden file exactly
//! (after whitespace normalization). Any difference means extraction
//! behavior changed and must be reviewed.
//!
//! To regenerate after intentional changes:
//!
//!     cargo test --test regression -- --ignored regenerate
//!
//! Then review with `git diff tests/expected/golden/`.

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

fn normalize(s: &str) -> String {
    s.lines()
        .map(|line| line.split_whitespace().collect::<Vec<_>>().join(" "))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn golden_files() {
    let golden = golden_dir();
    let mut failures = Vec::new();
    let mut total = 0;

    let mut missing_golden = Vec::new();

    for entry in fs::read_dir(fixtures_dir()).unwrap().flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("html") {
            continue;
        }
        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        let golden_path = golden.join(format!("{name}.html"));

        let Ok(expected) = fs::read_to_string(&golden_path) else {
            missing_golden.push(name);
            continue;
        };

        total += 1;
        let html = fs::read_to_string(&path).unwrap();
        let result = parse(&html, &DecruftOptions::default());

        if normalize(&result.content) != normalize(&expected) {
            failures.push(name);
        }
    }

    assert!(
        missing_golden.is_empty(),
        "fixtures missing golden files:\n  {}",
        missing_golden.join("\n  ")
    );
    assert!(
        failures.is_empty(),
        "{}/{total} golden mismatches:\n  {}\n\n\
         Regenerate: cargo test --test regression -- --ignored regenerate",
        failures.len(),
        failures.join("\n  ")
    );
}

#[test]
fn golden_markdown() {
    let golden = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/expected/golden-markdown");
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
        let mut opts = DecruftOptions::default();
        opts.markdown = true;
        let result = parse(&html, &opts);

        if normalize(&result.content) != normalize(&expected) {
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
        "golden markdown mismatches:\n  {}\n\n\
         Regenerate: cargo test --test regression -- --ignored regenerate",
        failures.join("\n  ")
    );
}

#[test]
#[ignore = "run manually to regenerate golden files"]
fn regenerate() {
    let fixtures = fixtures_dir();
    let golden = golden_dir();
    fs::create_dir_all(&golden).unwrap();

    let golden_md =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/expected/golden-markdown");
    fs::create_dir_all(&golden_md).unwrap();

    let mut count = 0;
    for entry in fs::read_dir(&fixtures).unwrap().flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("html") {
            continue;
        }
        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        let html = fs::read_to_string(&path).unwrap();

        let result = parse(&html, &DecruftOptions::default());
        fs::write(golden.join(format!("{name}.html")), &result.content).unwrap();

        let mut md_opts = DecruftOptions::default();
        md_opts.markdown = true;
        let md_result = parse(&html, &md_opts);
        fs::write(golden_md.join(format!("{name}.md")), &md_result.content).unwrap();

        count += 1;
    }

    panic!("Regenerated {count} golden files (HTML + markdown). Review: git diff tests/expected/");
}
