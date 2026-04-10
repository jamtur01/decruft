# AGENTS.md — instructions for AI coding agents

## Project context

decruft is a Rust port of [defuddle](https://github.com/kepano/defuddle) — a web content extraction library. It extracts clean, readable content and metadata from noisy HTML pages.

## Before making changes

1. Read the relevant source files before editing
2. Run `cargo test` to establish baseline (273 tests expected)
3. Check `cargo clippy --all-targets -- -D warnings` is clean

## After making changes

1. `cargo fmt`
2. `cargo clippy --all-targets -- -D warnings` — zero warnings required
3. `cargo test` — all tests must pass
4. If extraction behavior changed: `bash tests/compare_sites.sh`

## Code rules

- No `unwrap()` or `expect()` in non-test code
- Functions max 100 lines — split into helpers
- Clippy pedantic is enabled with `allow_attributes = "deny"`
- Use `let...else` for early returns, not `.unwrap()`
- Use `for` loops over iterator chains when clearer
- Char-safe string slicing only (`char_indices`, never raw byte offsets)

## Architecture rules

- Internal modules are `pub(crate)` — only `parse()`, `parse_with_defaults()`, `strip_html_tags()`, and types are public
- `DecruftOptions` and `DecruftResult` are `#[non_exhaustive]` — construct via `Default::default()` + field mutation
- Site extractors go in `src/extractors/{site}.rs` — follow the HTML-first, API-fallback pattern
- All CSS selectors live in `src/selectors.rs` — use `fancy_regex` for lookbehind patterns
- Scoring factors in `src/scorer.rs` must match defuddle's — don't change weights without justification

## Common pitfalls

- `scraper` crate doesn't support `:has()` CSS pseudo-class
- `ego-tree` doesn't support reparenting nodes easily — detach children first
- `fancy_regex::is_match()` returns `Result<bool>`, not `bool`
- HTML entities in JSON-LD need decoding (non-ASCII in `strip_json_comments`)
- `content_selector` option is a hard override — retries skip when it's set
- Network tests use `#[ignore]` — run with `cargo test -- --ignored`

## Test fixtures

- 144 HTML fixtures in `tests/fixtures/defuddle/` from defuddle's test suite
- 146 expected markdown files in `tests/expected/defuddle/`
- Our own fixtures in `tests/fixtures/` (complex_blog, news_article, wikipedia)
- Fixture-dependent tests panic if files are missing — all fixtures are checked in

## Network tests

Tests that make real HTTP requests are marked `#[ignore]` and only run with
`cargo test -- --ignored`. They should never run in CI. Each extractor with an
API fallback (GitHub, HN, Stack Overflow, Lobsters, C2 Wiki) also has mock-based
tests that validate the JSON parsing logic on canned data without network calls.
