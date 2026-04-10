# decruft

Extract clean, readable content from web pages. A Rust port of [defuddle](https://github.com/kepano/defuddle).

[![CI](https://github.com/jamtur01/decruft/actions/workflows/ci.yml/badge.svg)](https://github.com/jamtur01/decruft/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/decruft.svg)](https://crates.io/crates/decruft)
[![Docs](https://docs.rs/decruft/badge.svg)](https://docs.rs/decruft)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

## What it does

Given a noisy HTML page (ads, navigation, sidebars, popups, tracking pixels, cookie banners...), decruft extracts the main content and metadata:

- **Content** — the article/post body as clean HTML
- **Metadata** — title, author, published date, description, image, language, site name, favicon
- **Schema.org** — parsed JSON-LD data

## Install

```sh
cargo install decruft
```

Or add to your `Cargo.toml`:

```toml
[dependencies]
decruft = "0.1"
```

## CLI

```sh
# From stdin
curl -sL https://example.com/article | decruft --url https://example.com/article

# From file
decruft page.html --url https://example.com/page

# Output formats: json (default), html, text, markdown
decruft page.html -f html
decruft page.html -f text
decruft page.html -f markdown

# Fetch and extract
decruft -F --url https://example.com/article

# Debug mode (shows what was removed and why)
decruft page.html --debug | jq '.debug.removals'
```

### Options

```
Usage: decruft [OPTIONS] [INPUT]

Arguments:
  [INPUT]  Path to an HTML file to process. Use - for stdin [default: -]

Options:
  -u, --url <URL>             URL (for resolving relative URLs and metadata)
  -s, --selector <SELECTOR>   CSS selector to override content root detection
  -f, --format <FORMAT>       Output format: json, html, text, or markdown [default: json]
  -d, --debug                 Include removal details in output
  -F, --fetch                 Fetch the URL with curl before processing
      --markdown              Convert output to Markdown
      --no-images             Strip all images
      --no-exact-selectors    Disable exact CSS selector removal
      --no-partial-selectors  Disable partial class/id pattern removal
      --no-hidden             Disable hidden element removal
      --no-scoring            Disable content scoring removal
      --no-patterns           Disable content pattern removal
      --no-standardize        Disable content standardization
      --no-replies            Exclude replies/comments from extracted content
  -h, --help                  Print help
  -V, --version               Print version
```

## Library

```rust
use decruft::{parse, DecruftOptions};

let html = r#"<html>
  <head><title>My Article - Blog Name</title></head>
  <body>
    <nav><a href="/">Home</a></nav>
    <article>
      <h1>My Article</h1>
      <p>The actual content you want.</p>
    </article>
    <footer>Copyright 2025</footer>
  </body>
</html>"#;

let options = DecruftOptions {
    url: Some("https://example.com/article".into()),
    ..DecruftOptions::default()
};

let result = parse(html, &options);

assert_eq!(result.title, "My Article");
assert!(result.content.contains("actual content"));
assert!(!result.content.contains("Copyright"));
```

### What gets removed

| Category | Examples |
|----------|----------|
| **Ads** | `.ad`, `[data-ad-wrapper]`, `.adsense`, `.promo` |
| **Navigation** | `<nav>`, `.menu`, `.navbar`, `[role="navigation"]` |
| **Sidebars** | `<aside>`, `.sidebar`, `[role="complementary"]` |
| **Social** | `.share`, `.social`, share buttons, follow widgets |
| **Comments** | `#comments`, `.comments-section` |
| **Footers** | `<footer>`, copyright notices |
| **Popups** | `.modal`, `.overlay`, `.popup`, cookie banners |
| **Hidden** | `display:none`, `visibility:hidden`, `[hidden]` |
| **Metadata clutter** | Bylines, read time, breadcrumbs, tags, TOC |
| **Related content** | "You might also like", "More stories", card grids |
| **Newsletter CTAs** | Subscribe forms, email signup blocks |

### Extraction pipeline

1. Parse HTML and extract schema.org JSON-LD
2. Extract metadata (title, author, date, etc.) via priority chains across meta tags, schema.org, and DOM
3. Try site-specific extractors (GitHub, Reddit, Hacker News, X/Twitter, Substack, C2 Wiki, BBCode, AI chat conversations)
4. Find main content element using scored entry-point selectors
5. Standardize math, footnotes, callouts, and code blocks into canonical formats
6. Remove ads, navigation, sidebars, and other clutter via CSS selectors
7. Remove elements matching ~500 partial class/id patterns
8. Score and remove non-content blocks (link-dense, nav indicators)
9. Remove content patterns (bylines, read time, boilerplate, related posts)
10. Standardize output (clean attributes, normalize headings, resolve URLs, deduplicate images)
11. Retry with progressively relaxed filters if too little content was extracted

## Metadata priority chains

Each metadata field is extracted using a fallback chain:

- **Title**: `og:title` > `twitter:title` > schema.org `headline` > `<meta name="title">` > `<title>`
- **Author**: `<meta name="author">` > schema.org `author.name` > `[itemprop="author"]` > `.author`
- **Published**: schema.org `datePublished` > `article:published_time` > `<time>` element
- **Description**: `<meta name="description">` > `og:description` > `twitter:description` > schema.org
- **Image**: `og:image` > `twitter:image` > schema.org `image`
- **Language**: `<html lang>` > `content-language` meta > `og:locale`

## Acknowledgements

Inspired by [defuddle](https://github.com/kepano/defuddle) by Steph Ango. The selector lists, scoring heuristics, and extraction pipeline are adapted from defuddle's approach.

## License

MIT
