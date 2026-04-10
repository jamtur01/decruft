//! Integration tests for the `decruft` CLI binary.

#![allow(clippy::panic)]

use std::process::Command;

fn decruft() -> Command {
    Command::new(env!("CARGO_BIN_EXE_decruft"))
}

fn fixture_path(name: &str) -> String {
    format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"))
}

// ── Basic flags ─────────────────────────────────────────────────

#[test]
fn help_exits_zero_and_describes_tool() {
    let output = decruft()
        .arg("--help")
        .output()
        .expect("failed to run decruft --help");
    assert!(output.status.success(), "exit code: {:?}", output.status);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Extract clean"),
        "help should describe the tool: {stdout}"
    );
}

#[test]
fn version_exits_zero_and_contains_name() {
    let output = decruft()
        .arg("--version")
        .output()
        .expect("failed to run decruft --version");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("decruft"),
        "version should contain binary name: {stdout}"
    );
}

#[test]
fn invalid_flag_exits_nonzero() {
    let output = decruft()
        .arg("--no-such-flag")
        .output()
        .expect("failed to run decruft");
    assert!(
        !output.status.success(),
        "invalid flag should fail, got: {:?}",
        output.status
    );
}

// ── Input modes ─────────────────────────────────────────────────

#[test]
fn stdin_input_produces_json_with_word_count() {
    let html = "<html><body><article>\
                <p>Hello world from stdin input test.</p>\
                </article></body></html>";
    let output = decruft()
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .arg("-")
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            child
                .stdin
                .take()
                .expect("stdin")
                .write_all(html.as_bytes())
                .expect("write");
            child.wait_with_output()
        })
        .expect("failed to run decruft via stdin");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("word_count"),
        "JSON output should contain word_count: {stdout}"
    );
}

#[test]
fn file_input_produces_json() {
    let output = decruft()
        .arg(fixture_path("complex_blog.html"))
        .output()
        .expect("failed to run decruft on file");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("word_count"),
        "should produce JSON: {stdout}"
    );
    assert!(stdout.contains("Rust"), "should contain article content");
}

#[test]
fn bad_url_shows_error() {
    let output = decruft()
        .arg("https://localhost:1/nonexistent")
        .output()
        .expect("failed to run decruft with bad URL");
    assert!(!output.status.success(), "bad URL should fail");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Error") || stderr.contains("error"),
        "should report an error: {stderr}"
    );
}

// ── Output formats ──────────────────────────────────────────────

#[test]
fn format_html_produces_html() {
    let output = decruft()
        .args([&fixture_path("complex_blog.html"), "-f", "html"])
        .output()
        .expect("failed to run decruft -f html");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains('<'),
        "HTML format should contain angle brackets: {stdout}"
    );
}

#[test]
fn format_text_strips_tags() {
    let output = decruft()
        .args([&fixture_path("complex_blog.html"), "-f", "text"])
        .output()
        .expect("failed to run decruft -f text");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("<p>"),
        "text format should not contain <p> tags"
    );
    assert!(
        stdout.contains("Rust"),
        "text should contain article content"
    );
}

#[test]
fn format_markdown_has_md_syntax() {
    let output = decruft()
        .args([&fixture_path("complex_blog.html"), "-f", "markdown"])
        .output()
        .expect("failed to run decruft -f markdown");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains('#') || stdout.contains("**") || stdout.contains("- "),
        "markdown format should contain markdown syntax: {}",
        &stdout[..300.min(stdout.len())]
    );
}

// ── Feature flags ───────────────────────────────────────────────

#[test]
fn debug_includes_removals() {
    let output = decruft()
        .args([&fixture_path("complex_blog.html"), "--debug"])
        .output()
        .expect("failed to run decruft --debug");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("removals"),
        "debug JSON should include removals: {}",
        &stdout[..500.min(stdout.len())]
    );
}

#[test]
fn no_images_strips_images() {
    let output = decruft()
        .args([
            &fixture_path("news_article.html"),
            "-f",
            "html",
            "--no-images",
        ])
        .output()
        .expect("failed to run decruft --no-images");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("<img"), "no-images should strip img tags");
}

#[test]
fn selector_narrows_content() {
    let output = decruft()
        .args([
            &fixture_path("complex_blog.html"),
            "-f",
            "html",
            "-s",
            "article",
        ])
        .output()
        .expect("failed to run decruft --selector");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Rust"),
        "selector 'article' should still find content"
    );
}
