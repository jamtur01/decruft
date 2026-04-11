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

    for entry in fs::read_dir(fixtures_dir()).unwrap().flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("html") {
            continue;
        }
        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        let golden_path = golden.join(format!("{name}.html"));

        let Ok(expected) = fs::read_to_string(&golden_path) else {
            continue;
        };

        total += 1;
        let html = fs::read_to_string(&path).unwrap();
        let result = parse(&html, &DecruftOptions::default());

        if normalize(&result.content) != normalize(&expected) {
            failures.push(name);
        }
    }

    assert!(total >= 250, "expected ≥250 golden files, got {total}");
    assert!(
        failures.is_empty(),
        "{}/{total} golden mismatches:\n  {}\n\n\
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
    fs::create_dir_all(&golden).unwrap();

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
        count += 1;
    }

    panic!("Regenerated {count} golden files. Review: git diff tests/expected/golden/");
}
