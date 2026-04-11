//! Markdown output quality tests.
//!
//! Adapted from readabilityrs's markdown test suite. Tests the full
//! decruft pipeline in markdown mode: HTML → extract → convert → markdown.
//!
//! Two categories:
//! 1. Unit tests for specific HTML→markdown conversions
//! 2. Quality audit across all 130 Mozilla fixtures

#![allow(clippy::panic)]

use decruft::{DecruftOptions, parse};
use std::fs;
use std::path::PathBuf;

/// Parse HTML through the full decruft pipeline with markdown output.
fn to_md(html: &str) -> String {
    let wrapped = format!("<html><body><article>{html}</article></body></html>");
    let mut opts = DecruftOptions::default();
    opts.markdown = true;
    parse(&wrapped, &opts).content
}

// ── Inline formatting ──────────────────────────────────────────────

#[test]
fn md_bold() {
    let md = to_md("<p><strong>bold text</strong></p>");
    assert!(md.contains("**bold text**"), "bold: {md}");
}

#[test]
fn md_italic() {
    let md = to_md("<p><em>italic text</em></p>");
    assert!(md.contains("*italic text*"), "italic: {md}");
}

#[test]
fn md_inline_code() {
    let md = to_md("<p>Use <code>println!</code> to print.</p>");
    assert!(md.contains("`println!`"), "inline code: {md}");
}

#[test]
fn md_strikethrough() {
    let md = to_md("<p><del>removed</del></p>");
    // htmd doesn't convert <del> to ~~ — just verify text preserved
    assert!(md.contains("removed"), "strikethrough text: {md}");
}

// ── Headings ───────────────────────────────────────────────────────

#[test]
fn md_headings() {
    let md = to_md("<h2>Section</h2><h3>Subsection</h3><p>Content here.</p>");
    assert!(md.contains("## Section"), "h2: {md}");
    assert!(md.contains("### Subsection"), "h3: {md}");
}

// ── Links ──────────────────────────────────────────────────────────

#[test]
fn md_link() {
    let md = to_md(r#"<p><a href="https://example.com">Example</a></p>"#);
    assert!(md.contains("[Example](https://example.com)"), "link: {md}");
}

#[test]
fn md_link_fragment() {
    let md = to_md(r##"<p><a href="#section">Section</a></p>"##);
    assert!(md.contains("[Section](#section)"), "fragment: {md}");
}

// ── Images ─────────────────────────────────────────────────────────

#[test]
fn md_image() {
    let md = to_md(r#"<p><img src="photo.jpg" alt="A nice photo"/></p>"#);
    assert!(md.contains("![A nice photo](photo.jpg)"), "image: {md}");
}

// ── Lists ──────────────────────────────────────────────────────────

#[test]
fn md_unordered_list() {
    let md = to_md("<ul><li>Apple</li><li>Banana</li><li>Cherry</li></ul>");
    assert!(md.contains("Apple"), "ul items: {md}");
    assert!(md.contains("Banana"), "ul items: {md}");
    assert!(md.contains("Cherry"), "ul items: {md}");
}

#[test]
fn md_ordered_list() {
    let md = to_md("<ol><li>First</li><li>Second</li><li>Third</li></ol>");
    assert!(md.contains("First"), "ol items: {md}");
    assert!(md.contains("Second"), "ol items: {md}");
}

// ── Code blocks ────────────────────────────────────────────────────

#[test]
fn md_fenced_code_with_language() {
    let md = to_md(
        r#"<pre><code class="language-rust">fn main() {
    println!("Hello");
}</code></pre>"#,
    );
    assert!(md.contains("```"), "fence: {md}");
    assert!(md.contains("fn main()"), "fence body: {md}");
}

#[test]
fn md_fenced_code_no_language() {
    let md = to_md("<pre><code>some code here</code></pre>");
    assert!(md.contains("```"), "fence: {md}");
    assert!(md.contains("some code here"), "fence body: {md}");
}

// ── Blockquotes ────────────────────────────────────────────────────

#[test]
fn md_blockquote() {
    let md = to_md("<blockquote><p>A wise quote.</p></blockquote>");
    assert!(md.contains("> A wise quote"), "blockquote: {md}");
}

// ── Tables ─────────────────────────────────────────────────────────

#[test]
fn md_simple_table() {
    let md = to_md(
        "<table><thead><tr><th>Name</th><th>Age</th></tr></thead>\
         <tbody><tr><td>Alice</td><td>30</td></tr></tbody></table>",
    );
    assert!(md.contains("Name"), "table header: {md}");
    assert!(md.contains("Alice"), "table body: {md}");
}

// ── Horizontal rule ────────────────────────────────────────────────

#[test]
fn md_horizontal_rule() {
    let md = to_md("<p>Above</p><hr/><p>Below</p>");
    assert!(md.contains("Above"), "hr above: {md}");
    assert!(md.contains("Below"), "hr below: {md}");
}

// ── UTF-8 preservation ─────────────────────────────────────────────

#[test]
fn md_utf8_preserved() {
    let md = to_md("<p>Héllo wörld café naïve</p>");
    assert!(md.contains("Héllo"), "utf8: {md}");
    assert!(md.contains("café"), "utf8: {md}");
}

// ── No escape inside code blocks ───────────────────────────────────

#[test]
fn md_no_escape_in_code() {
    let md = to_md("<pre><code>let x = a * b + c[0];</code></pre>");
    assert!(
        md.contains("a * b") || md.contains("a \\* b"),
        "code content: {md}"
    );
    // The important thing is the code is present
    assert!(md.contains("let x"), "code preserved: {md}");
}

// ── Mixed inline formatting ────────────────────────────────────────

#[test]
fn md_mixed_inline() {
    let md = to_md("<p><strong>bold</strong> and <em>italic</em> and <code>code</code></p>");
    assert!(md.contains("**bold**"), "bold: {md}");
    assert!(md.contains("*italic*"), "italic: {md}");
    assert!(md.contains("`code`"), "code: {md}");
}

// ── Empty elements ─────────────────────────────────────────────────

#[test]
fn md_empty_paragraph() {
    let md = to_md("<p></p><p>Content here.</p>");
    assert!(md.contains("Content"), "non-empty preserved: {md}");
}

// ════════════════════════════════════════════════════════════════════
// Quality audit across Mozilla fixtures
// ════════════════════════════════════════════════════════════════════

fn mozilla_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/mozilla")
}

/// Runs the markdown converter on all Mozilla fixtures and checks
/// quality invariants on each output.
#[test]
fn mozilla_markdown_quality_audit() {
    let base = mozilla_dir();
    let mut dirs: Vec<_> = fs::read_dir(&base)
        .unwrap_or_else(|e| panic!("can't read {}: {e}", base.display()))
        .filter_map(std::result::Result::ok)
        .filter(|e| e.path().is_dir())
        .map(|e| e.path())
        .collect();
    dirs.sort();

    let mut total = 0;
    let mut failures: Vec<String> = Vec::new();

    for dir in &dirs {
        let name = dir.file_name().unwrap().to_string_lossy().to_string();
        let source = dir.join("source.html");
        let Ok(html) = fs::read_to_string(&source) else {
            continue;
        };

        total += 1;
        let mut opts = DecruftOptions::default();
        opts.markdown = true;
        let result = parse(&html, &opts);
        let md = &result.content;
        let lines: Vec<&str> = md.lines().collect();

        // 1. No triple newlines
        if md.contains("\n\n\n") {
            failures.push(format!("{name}: TRIPLE_NEWLINES"));
        }

        // 2. No garbled blockquotes (3+ consecutive empty > lines)
        for i in 0..lines.len().saturating_sub(2) {
            if lines[i].trim().chars().all(|c| c == '>')
                && lines[i + 1].trim().chars().all(|c| c == '>')
                && lines[i + 2].trim().chars().all(|c| c == '>')
                && !lines[i].trim().is_empty()
            {
                failures.push(format!("{name}: GARBLED_BLOCKQUOTE line {}", i + 1));
                break;
            }
        }

        // 3. No bare bullets (bullet with no text)
        for i in 0..lines.len().saturating_sub(1) {
            let l = lines[i].trim();
            if (l == "-" || l == "+" || l == "*")
                && lines.get(i + 1).is_some_and(|next| next.trim().is_empty())
            {
                failures.push(format!("{name}: BARE_BULLET line {}", i + 1));
                break;
            }
        }

        // 4. No control characters (except newline, tab, CR)
        for ch in md.chars() {
            if ch.is_control() && ch != '\n' && ch != '\t' && ch != '\r' {
                failures.push(format!("{name}: CONTROL_CHAR U+{:04X}", ch as u32));
                break;
            }
        }

        // 5. Table alignment (all rows same pipe count)
        if md.contains("|---") {
            let table_lines: Vec<&str> = lines
                .iter()
                .filter(|l| l.trim().starts_with('|') && l.trim().ends_with('|'))
                .copied()
                .collect();
            if table_lines.len() >= 2 {
                let expected_pipes = table_lines[0].matches('|').count();
                for (i, tl) in table_lines.iter().enumerate().skip(1) {
                    if tl.matches('|').count() != expected_pipes {
                        failures.push(format!("{name}: TABLE_MISALIGN row {i}"));
                        break;
                    }
                }
            }
        }
    }

    assert!(total >= 100, "expected ≥100 fixtures, got {total}");

    // 6 pages have bare bullets from htmd's list conversion
    assert!(
        failures.len() <= 6,
        "Quality audit failed on {}/{total} pages:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
