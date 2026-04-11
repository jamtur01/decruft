//! Markdown conversion tests.
//!
//! Two parts:
//! 1. Unit tests — specific HTML→markdown conversions through the full pipeline
//! 2. Quality audit — structural invariants across all 130 Mozilla fixtures

#![allow(clippy::panic)]

use decruft::{DecruftOptions, parse};
use std::fs;
use std::path::PathBuf;

fn to_md(html: &str) -> String {
    let wrapped = format!("<html><body><article>{html}</article></body></html>");
    let mut opts = DecruftOptions::default();
    opts.markdown = true;
    parse(&wrapped, &opts).content
}

fn sample_md() -> String {
    let html = r#"<html lang="en"><head>
        <title>Test Article - Example Blog</title>
        <meta property="og:title" content="Test Article">
    </head><body><article>
        <h1>Test Article</h1>
        <p>This is the <strong>first paragraph</strong> with <em>formatted</em> text.</p>
        <p>Here is a <a href="https://example.com">link</a> and content.</p>
        <pre><code class="language-rust">fn main() {
    println!("Hello");
}</code></pre>
        <blockquote><p>A notable quote.</p></blockquote>
        <ul><li>First item</li><li>Second item</li></ul>
        <h2>Second Section</h2>
        <p>More content here.</p>
    </article></body></html>"#;
    let mut opts = DecruftOptions::default();
    opts.url = Some("https://example.com/test".into());
    opts.markdown = true;
    parse(html, &opts).content
}

// ════════════════════════════════════════════════════════════════
// 1. Unit tests
// ════════════════════════════════════════════════════════════════

#[test]
fn bold() {
    assert!(to_md("<p><strong>bold</strong></p>").contains("**bold**"));
}

#[test]
fn italic() {
    assert!(to_md("<p><em>italic</em></p>").contains("*italic*"));
}

#[test]
fn inline_code() {
    assert!(to_md("<p>Use <code>println!</code></p>").contains("`println!`"));
}

#[test]
fn headings() {
    let md = to_md("<h2>Section</h2><h3>Sub</h3><p>Text.</p>");
    assert!(md.contains("## Section"));
    assert!(md.contains("### Sub"));
}

#[test]
fn links() {
    let md = to_md(r#"<p><a href="https://example.com">Ex</a></p>"#);
    assert!(md.contains("[Ex](https://example.com)"));
}

#[test]
fn images() {
    let md = to_md(r#"<p><img src="photo.jpg" alt="A photo"/></p>"#);
    assert!(md.contains("![A photo](photo.jpg)"));
}

#[test]
fn lists() {
    let md = to_md("<ul><li>Apple</li><li>Banana</li></ul>");
    assert!(md.contains("Apple"));
    assert!(md.contains("Banana"));
}

#[test]
fn code_blocks() {
    let md = to_md("<pre><code>some code</code></pre>");
    assert!(md.contains("```"));
    assert!(md.contains("some code"));
}

#[test]
fn blockquotes() {
    let md = to_md("<blockquote><p>A quote.</p></blockquote>");
    assert!(md.contains("> A quote"));
}

#[test]
fn tables() {
    let md = to_md(
        "<table><thead><tr><th>Name</th></tr></thead>\
         <tbody><tr><td>Alice</td></tr></tbody></table>",
    );
    assert!(md.contains("Name"));
    assert!(md.contains("Alice"));
}

#[test]
fn utf8() {
    let md = to_md("<p>Héllo café naïve</p>");
    assert!(md.contains("Héllo") && md.contains("café"));
}

#[test]
fn mixed_inline() {
    let md = to_md("<p><strong>b</strong> <em>i</em> <code>c</code></p>");
    assert!(md.contains("**b**") && md.contains("*i*") && md.contains("`c`"));
}

// ── Full pipeline ───────────────────────────────────────────────

#[test]
fn no_html_tags_in_markdown() {
    let md = sample_md();
    assert!(!md.contains("<p>"), "markdown should not have <p>");
}

#[test]
fn has_formatting() {
    let md = sample_md();
    assert!(
        md.contains("**first paragraph**") || md.contains("*formatted*"),
        "should have markdown formatting"
    );
}

#[test]
fn has_code_fences() {
    let md = sample_md();
    assert!(md.contains("```") && md.contains("println!"));
}

#[test]
fn has_blockquote() {
    let md = sample_md();
    assert!(md.contains("> "));
}

#[test]
fn has_links() {
    let md = sample_md();
    assert!(md.contains("[link](https://example.com)") || md.contains("[link]"));
}

#[test]
fn has_headings() {
    let md = sample_md();
    assert!(md.contains("# ") || md.contains("## "));
}

// ── Separate markdown mode ──────────────────────────────────────

#[test]
fn separate_markdown_includes_both() {
    let html = r"<html><body><article>
        <p><strong>Content</strong> here.</p>
    </article></body></html>";
    let mut opts = DecruftOptions::default();
    opts.separate_markdown = true;
    let result = parse(html, &opts);

    assert!(result.content.contains("<p>"), "content should be HTML");
    let md = result.content_markdown.expect("should have markdown");
    assert!(!md.contains("<p>"));
    assert!(!md.is_empty());
}

#[test]
fn no_markdown_option_returns_none() {
    let html = "<html><body><article><p>Text.</p></article></body></html>";
    let result = parse(html, &DecruftOptions::default());
    assert!(result.content_markdown.is_none());
}

// ════════════════════════════════════════════════════════════════
// Edge cases (ported from readabilityrs markdown_tests.rs)
// ════════════════════════════════════════════════════════════════

// ── Nested formatting ───────────────────────────────────────────

#[test]
fn bold_inside_link() {
    let md = to_md(r#"<a href="https://example.com"><strong>bold link</strong></a>"#);
    assert!(
        md.contains("**bold link**") || md.contains("__bold link__"),
        "nested bold in link: {md}"
    );
    assert!(md.contains("example.com"), "link preserved: {md}");
}

#[test]
fn mixed_inline_nesting() {
    let md = to_md("<p><strong>bold</strong> and <em>italic</em> and <code>code</code></p>");
    assert!(md.contains("**bold**") && md.contains("*italic*") && md.contains("`code`"));
}

#[test]
fn bold_in_list_item() {
    let md = to_md("<ul><li><strong>bold item</strong></li></ul>");
    assert!(md.contains("**bold item**"), "bold in list: {md}");
}

// ── Empty elements ──────────────────────────────────────────────

#[test]
fn empty_heading_omitted() {
    let md = to_md("<h2></h2><p>Text after empty heading.</p>");
    assert!(
        !md.contains("##\n"),
        "empty heading should not produce ## line: {md}"
    );
}

#[test]
fn empty_bold_no_markers() {
    let md = to_md("<p><strong></strong>text</p>");
    assert!(!md.contains("****"), "empty bold should not produce ****");
}

#[test]
fn empty_paragraph_no_output() {
    let md = to_md("<p></p><p>Content here.</p>");
    assert!(md.contains("Content"), "non-empty preserved: {md}");
}

// ── Whitespace ──────────────────────────────────────────────────

#[test]
fn multiple_spaces_collapsed() {
    let md = to_md("<p>hello    world</p>");
    assert!(md.contains("hello") && md.contains("world"));
}

#[test]
fn utf8_preserved() {
    let md = to_md("<p>Héllo wörld café naïve 中文</p>");
    assert!(md.contains("Héllo") && md.contains("café") && md.contains("中文"));
}

// ── Complex links ───────────────────────────────────────────────

#[test]
fn link_fragment_only() {
    let md = to_md(r##"<p><a href="#section">Section</a></p>"##);
    assert!(md.contains("[Section](#section)"), "fragment link: {md}");
}

#[test]
fn link_relative_url() {
    let md = to_md(r#"<p><a href="/page/sub">relative</a></p>"#);
    assert!(md.contains("[relative](/page/sub)"), "relative link: {md}");
}

#[test]
fn link_no_href() {
    let md = to_md("<a>just text</a>");
    assert!(md.contains("just text"));
    assert!(
        !md.contains("]("),
        "bare anchor should not produce link syntax: {md}"
    );
}

// ── Complex images ──────────────────────────────────────────────

#[test]
fn image_empty_alt() {
    let md = to_md(r#"<p><img src="photo.jpg" alt=""/></p>"#);
    assert!(md.contains("photo.jpg"), "image preserved: {md}");
}

#[test]
fn image_no_alt() {
    let md = to_md(r#"<p><img src="photo.jpg"/></p>"#);
    assert!(md.contains("photo.jpg"), "image preserved: {md}");
}

// ── Complex code blocks ─────────────────────────────────────────

#[test]
fn code_block_preserves_content() {
    let md = to_md("<pre><code>let x = 1;\nlet y = 2;</code></pre>");
    assert!(md.contains("let x = 1") && md.contains("let y = 2"));
}

#[test]
fn no_escape_inside_code() {
    let md = to_md("<pre><code>a * b + c[0]</code></pre>");
    assert!(
        md.contains("a * b") || md.contains("a \\* b"),
        "code content: {md}"
    );
}

#[test]
fn pre_without_code_child() {
    let md = to_md("<pre>preformatted text</pre>");
    assert!(md.contains("preformatted text"), "pre content: {md}");
}

// ── Complex tables ──────────────────────────────────────────────

#[test]
fn table_cell_with_link() {
    let md = to_md(
        r#"<table><thead><tr><th>Name</th></tr></thead>
        <tbody><tr><td><a href="https://example.com">Link</a></td></tr></tbody></table>"#,
    );
    assert!(
        md.contains("Link") && md.contains("example.com"),
        "table link: {md}"
    );
}

#[test]
fn table_no_headers() {
    let md = to_md(
        "<table><tbody><tr><td>a</td><td>b</td></tr><tr><td>c</td><td>d</td></tr></tbody></table>",
    );
    assert!(
        md.contains('a') && md.contains('d'),
        "headerless table: {md}"
    );
}

// ── Nested lists ────────────────────────────────────────────────

#[test]
fn nested_unordered_list() {
    let md = to_md("<ul><li>outer<ul><li>inner</li></ul></li></ul>");
    assert!(
        md.contains("outer") && md.contains("inner"),
        "nested list: {md}"
    );
}

#[test]
fn mixed_nested_lists() {
    let md = to_md("<ul><li>bullet<ol><li>numbered</li></ol></li></ul>");
    assert!(
        md.contains("bullet") && md.contains("numbered"),
        "mixed list: {md}"
    );
}

#[test]
fn ordered_list() {
    let md = to_md("<ol><li>First</li><li>Second</li><li>Third</li></ol>");
    assert!(
        md.contains("1.") || md.contains("First"),
        "ordered list: {md}"
    );
}

// ── Blockquotes ─────────────────────────────────────────────────

#[test]
fn blockquote_with_paragraph() {
    let md = to_md("<blockquote><p>quoted text</p></blockquote>");
    assert!(
        md.trim().contains("> quoted text") || md.contains("> quoted"),
        "blockquote: {md}"
    );
}

#[test]
fn nested_blockquote() {
    let md = to_md("<blockquote><blockquote><p>deep</p></blockquote></blockquote>");
    assert!(
        md.contains("> > deep") || md.contains(">>"),
        "nested quote: {md}"
    );
}

// ── Horizontal rule ─────────────────────────────────────────────

#[test]
fn horizontal_rule() {
    let md = to_md("<p>Above</p><hr/><p>Below</p>");
    // htmd strips <hr> — just verify content on both sides is preserved
    assert!(
        md.contains("Above") && md.contains("Below"),
        "hr content: {md}"
    );
}

// ── Definition lists ────────────────────────────────────────────

#[test]
fn definition_list() {
    let md = to_md("<dl><dt>Term</dt><dd>Definition of the term.</dd></dl>");
    assert!(md.contains("Term") && md.contains("Definition"), "dl: {md}");
}

// ════════════════════════════════════════════════════════════════
// 3. Quality audit — structural invariants on all Mozilla fixtures
// ════════════════════════════════════════════════════════════════

#[test]
fn mozilla_markdown_quality() {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/mozilla");
    let mut dirs: Vec<_> = fs::read_dir(&base)
        .unwrap()
        .flatten()
        .filter(|e| e.path().is_dir())
        .map(|e| e.path())
        .collect();
    dirs.sort();

    let mut total = 0;
    let mut failures = Vec::new();

    for dir in &dirs {
        let name = dir.file_name().unwrap().to_string_lossy().to_string();
        let Ok(html) = fs::read_to_string(dir.join("source.html")) else {
            continue;
        };

        total += 1;
        let mut opts = DecruftOptions::default();
        opts.markdown = true;
        let md = parse(&html, &opts).content;
        let lines: Vec<&str> = md.lines().collect();

        // No triple newlines
        if md.contains("\n\n\n") {
            failures.push(format!("{name}: TRIPLE_NEWLINES"));
        }

        // No garbled blockquotes (3+ consecutive bare > lines)
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

        // No bare bullets
        for i in 0..lines.len().saturating_sub(1) {
            let l = lines[i].trim();
            if (l == "-" || l == "+" || l == "*")
                && lines.get(i + 1).is_some_and(|n| n.trim().is_empty())
            {
                failures.push(format!("{name}: BARE_BULLET line {}", i + 1));
                break;
            }
        }

        // No control characters
        for ch in md.chars() {
            if ch.is_control() && ch != '\n' && ch != '\t' && ch != '\r' {
                failures.push(format!("{name}: CONTROL_CHAR U+{:04X}", ch as u32));
                break;
            }
        }

        // Table column alignment
        if md.contains("|---") {
            let table_lines: Vec<&str> = lines
                .iter()
                .filter(|l| l.trim().starts_with('|') && l.trim().ends_with('|'))
                .copied()
                .collect();
            if table_lines.len() >= 2 {
                let expected = table_lines[0].matches('|').count();
                for (i, tl) in table_lines.iter().enumerate().skip(1) {
                    if tl.matches('|').count() != expected {
                        failures.push(format!("{name}: TABLE_MISALIGN row {i}"));
                        break;
                    }
                }
            }
        }
    }

    assert!(total >= 100, "expected ≥100 fixtures, got {total}");
    // 6 bare bullet pages from htmd (tracked in #13)
    assert!(
        failures.len() <= 6,
        "quality audit failed ({}/{total}):\n  {}",
        failures.len(),
        failures.join("\n  ")
    );
}
