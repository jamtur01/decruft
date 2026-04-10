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

3. If you changed extraction behavior, run the comparison script to verify no regressions:
   ```sh
   bash tests/compare_sites.sh
   ```

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
6. Add the site to `tests/compare_sites.sh`

## Reporting bugs

Use [GitHub Issues](https://github.com/jamtur01/decruft/issues). Include:
- The URL or HTML that produces wrong output
- What you expected vs what you got
- `decruft --version` output
