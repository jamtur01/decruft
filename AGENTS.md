# AGENTS.md — instructions for AI coding agents

## Project context

decruft is a Rust port of [defuddle](https://github.com/kepano/defuddle) — a web content extraction library. It extracts clean, readable content and metadata from noisy HTML pages.

## Before making changes

1. Read the relevant source files before editing
2. Run `cargo test` to establish baseline
3. Check `cargo clippy --all-targets -- -D warnings` is clean

## After making changes

1. `cargo fmt`
2. `cargo clippy --all-targets -- -D warnings` — zero warnings required
3. `cargo test` — all tests must pass, zero failures allowed
4. If extraction behavior changed:
   - `cargo test --test regression -- --ignored regenerate` to update golden files
   - `cargo test --test extraction -- --ignored regenerate_metadata` to update metadata
   - Review diffs with `git diff tests/expected/`
   - `bash tests/compare_sites.sh` for comparison vs defuddle

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

## Test structure

Six test files, each with a single concern:

- `tests/regression.rs` — golden file exact match (HTML + markdown), byte-for-byte, with fixture URLs
- `tests/extraction.rs` — metadata exact match against JSON expectations + non-empty sweep
- `tests/behavior.rs` — pipeline option toggles (ported from defuddle)
- `tests/markdown.rs` — markdown conversion unit tests + zero-tolerance quality audit
- `tests/formats.rs` — JSON/HTML/text output + public API coverage
- `tests/cli.rs` — CLI binary tests

## Test fixtures

- 280 HTML fixtures in `tests/fixtures/` (flat — 144 from defuddle, 130 from Mozilla, 6 standalone)
- 280 golden HTML files in `tests/expected/golden/`
- 280 golden markdown files in `tests/expected/golden-markdown/`
- 280 metadata JSON files in `tests/expected/metadata/`
- Every fixture MUST have all three expected files — tests fail on missing files
- Mozilla fixtures are prefixed `mozilla--` (e.g. `mozilla--bbc-1.html`)

## Network tests

Most network tests (GitHub, HN, Stack Overflow, Lobsters, C2 Wiki) run by default.
The X/Twitter oEmbed test is `#[ignore]` because the API is frequently rate-limited.
Run with `cargo test -- --ignored` to include it.
