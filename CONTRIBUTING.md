# Contributing to decruft

Thanks for your interest in contributing.

## Getting started

```sh
git clone https://github.com/jamtur01/decruft.git
cd decruft
cargo test
```

## Before submitting a PR

1. Run the full check suite:
   ```sh
   cargo fmt --check
   cargo clippy --all-targets -- -D warnings
   cargo test
   ```

2. If you added a new feature, add tests for it.

3. If you changed extraction behavior:
   ```sh
   # Regenerate golden files and metadata
   cargo test --test regression -- --ignored regenerate
   cargo test --test extraction -- --ignored regenerate_metadata
   # Review what changed
   git diff tests/expected/
   # Compare against defuddle
   bash tests/compare_sites.sh
   ```

## Test structure

280 fixtures in `tests/fixtures/` (flat). Each has three expected files:
- `tests/expected/golden/{name}.html` — golden HTML output (byte-exact match)
- `tests/expected/golden-markdown/{name}.md` — golden markdown output (byte-exact match)
- `tests/expected/metadata/{name}.json` — metadata expectations (exact field match)

Six test files:
- `tests/regression.rs` — golden file exact match (primary regression gate)
- `tests/extraction.rs` — metadata exact match + non-empty extraction sweep
- `tests/behavior.rs` — pipeline option toggles (ported from defuddle)
- `tests/markdown.rs` — markdown conversion + quality audit
- `tests/formats.rs` — JSON/HTML/text output + public API
- `tests/cli.rs` — CLI binary tests

Zero tolerance: every test passes or fails. No thresholds, no budgets, no allowed failures.

## Code style

- Functions under 100 lines
- No `unwrap()` or `expect()` outside of tests
- Clippy pedantic enabled — fix all warnings
- Use `for` loops over long iterator chains when clearer

## Adding a site-specific extractor

1. Create `src/extractors/{site}.rs`
2. Implement detection (`is_{site}`) and extraction (`extract_{site}`)
3. Follow the HTML-first, API-fallback pattern (see `github.rs` or `hackernews.rs`)
4. Register in `src/extractors/mod.rs`
5. Add unit tests in the file
6. Add fixture HTML to `tests/fixtures/` with a URL comment: `<!-- {"url":"https://..."} -->`
7. Run regeneration to create golden + metadata expected files
8. Add the site to `tests/compare_sites.sh`

## Reporting bugs

Use [GitHub Issues](https://github.com/jamtur01/decruft/issues). Include:
- The URL or HTML that produces wrong output
- What you expected vs what you got
- `decruft --version` output
