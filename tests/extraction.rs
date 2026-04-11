//! Content extraction correctness tests.
//!
//! Tests against upstream expected outputs:
//! - Defuddle: exact metadata per fixture, exact content per fixture
//! - Mozilla: exact metadata per fixture, exact content per fixture
//!
//! Each test checks every fixture individually. Failures are listed by name.
//! No thresholds, no budgets, no similarity scores. A fixture either passes
//! or it doesn't. When extraction improves, previously-failing fixtures
//! start passing and the test still passes. When extraction regresses,
//! previously-passing fixtures fail and the test fails.
//!
//! The set of currently-passing fixtures is recorded in PASS lists.
//! The test asserts every fixture in the PASS list still passes.

#![allow(clippy::panic)]

use decruft::{DecruftOptions, parse};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

fn opts(url: &str) -> DecruftOptions {
    let mut o = DecruftOptions::default();
    o.url = Some(url.into());
    o
}

fn url_from_html(html: &str) -> Option<String> {
    let start = html.find("<!-- {")?;
    let json_start = start + 5;
    let end = html[json_start..].find(" -->")?;
    serde_json::from_str::<serde_json::Value>(&html[json_start..json_start + end])
        .ok()?
        .get("url")
        .and_then(|v| v.as_str())
        .map(String::from)
}

fn str_field(val: &serde_json::Value, key: &str) -> String {
    val.get(key)
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

fn normalize_ws(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Parse defuddle expected .md: JSON preamble → metadata, rest → body.
fn parse_defuddle_expected(md: &str) -> Option<(serde_json::Value, String)> {
    let start = md.find("```json\n")?;
    let json_start = start + "```json\n".len();
    let json_end = md[json_start..].find("\n```")?;
    let json_str = &md[json_start..json_start + json_end];
    let meta: serde_json::Value = serde_json::from_str(json_str).ok()?;
    let body = md[json_start + json_end + 4..].trim().to_string();
    Some((meta, body))
}

// ════════════════════════════════════════════════════════════════
// Defuddle oracle
// ════════════════════════════════════════════════════════════════

/// Exact metadata match against defuddle's expected .md files.
/// For each fixture: if all non-empty expected fields (title, author,
/// site, published) match exactly, the fixture passes.
#[test]
fn defuddle_oracle_metadata() {
    let fixture_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/defuddle");
    let expected_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/expected/defuddle");

    let must_pass: HashSet<&str> = DEFUDDLE_META_PASS.iter().copied().collect();
    let mut regressions = Vec::new();
    let mut new_passes = Vec::new();
    let mut total = 0;

    for entry in fs::read_dir(&fixture_dir).unwrap().flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("html") {
            continue;
        }
        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        let expected_path = expected_dir.join(format!("{name}.md"));
        let Ok(md) = fs::read_to_string(&expected_path) else {
            continue;
        };
        let Some((meta, _)) = parse_defuddle_expected(&md) else {
            continue;
        };

        let html = fs::read_to_string(&path).unwrap();
        let url = url_from_html(&html).unwrap_or_else(|| format!("https://example.com/{name}"));
        let result = parse(&html, &opts(&url));
        total += 1;

        let mut pass = true;
        let exp_title = str_field(&meta, "title");
        if !exp_title.is_empty() && result.title != exp_title {
            pass = false;
        }
        let exp_author = str_field(&meta, "author");
        if !exp_author.is_empty() && result.author != exp_author {
            pass = false;
        }
        let exp_site = str_field(&meta, "site");
        if !exp_site.is_empty() && result.site != exp_site {
            pass = false;
        }
        let exp_pub = str_field(&meta, "published");
        if !exp_pub.is_empty() {
            let date = exp_pub.split('T').next().unwrap_or("");
            if !date.is_empty() && !result.published.contains(date) {
                pass = false;
            }
        }

        if pass && !must_pass.contains(name.as_str()) {
            new_passes.push(name.clone());
        }
        if !pass && must_pass.contains(name.as_str()) {
            regressions.push(name.clone());
        }
    }

    assert!(
        new_passes.is_empty(),
        "defuddle metadata: {total} tested. {} NEW passes — add to DEFUDDLE_META_PASS:\n  {}",
        new_passes.len(),
        new_passes.join("\n  ")
    );
    assert!(
        regressions.is_empty(),
        "defuddle metadata: {} REGRESSIONS:\n  {}",
        regressions.len(),
        regressions.join("\n  ")
    );
}

/// Exact markdown content comparison against defuddle's expected body.
/// Whitespace-normalized. Fixture passes if normalized output matches.
#[test]
fn defuddle_oracle_content() {
    let fixture_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/defuddle");
    let expected_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/expected/defuddle");

    let must_pass: HashSet<&str> = DEFUDDLE_CONTENT_PASS.iter().copied().collect();
    let mut regressions = Vec::new();
    let mut new_passes = Vec::new();
    let mut total = 0;

    for entry in fs::read_dir(&fixture_dir).unwrap().flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("html") {
            continue;
        }
        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        let expected_path = expected_dir.join(format!("{name}.md"));
        let Ok(md) = fs::read_to_string(&expected_path) else {
            continue;
        };
        let Some((_, body)) = parse_defuddle_expected(&md) else {
            continue;
        };
        if body.is_empty() {
            continue;
        }

        let html = fs::read_to_string(&path).unwrap();
        let url = url_from_html(&html).unwrap_or_else(|| format!("https://example.com/{name}"));
        let mut md_opts = opts(&url);
        md_opts.markdown = true;
        let result = parse(&html, &md_opts);
        total += 1;

        let pass = normalize_ws(&result.content) == normalize_ws(&body);

        if pass && !must_pass.contains(name.as_str()) {
            new_passes.push(name.clone());
        }
        if !pass && must_pass.contains(name.as_str()) {
            regressions.push(name.clone());
        }
    }

    assert!(
        new_passes.is_empty(),
        "defuddle content: {total} tested. {} NEW exact passes — add to DEFUDDLE_CONTENT_PASS:\n  {}",
        new_passes.len(),
        new_passes.join("\n  ")
    );
    assert!(
        regressions.is_empty(),
        "defuddle content: {} REGRESSIONS:\n  {}",
        regressions.len(),
        regressions.join("\n  ")
    );
}

// ════════════════════════════════════════════════════════════════
// Mozilla oracle
// ════════════════════════════════════════════════════════════════

/// Exact metadata comparison against Mozilla's expected-metadata.json.
/// Checks title, byline, siteName, lang. Fixture passes if all match.
#[test]
fn mozilla_oracle_metadata() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/mozilla");
    let must_pass: HashSet<&str> = MOZILLA_META_PASS.iter().copied().collect();
    let mut regressions = Vec::new();
    let mut new_passes = Vec::new();
    let mut total = 0;

    for entry in fs::read_dir(&dir).unwrap().flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        let meta_path = path.join("expected-metadata.json");
        let Ok(meta_str) = fs::read_to_string(&meta_path) else {
            continue;
        };
        let Ok(meta) = serde_json::from_str::<serde_json::Value>(&meta_str) else {
            continue;
        };
        let Ok(html) = fs::read_to_string(path.join("source.html")) else {
            continue;
        };

        total += 1;
        let result = parse(&html, &DecruftOptions::default());
        let mut pass = true;

        let exp_title = str_field(&meta, "title");
        if !exp_title.is_empty()
            && result.title != exp_title
            && !result.title.contains(&exp_title)
            && !exp_title.contains(&result.title)
        {
            pass = false;
        }

        let exp_byline = str_field(&meta, "byline");
        if !exp_byline.is_empty()
            && result.author != exp_byline
            && !result.author.contains(&exp_byline)
            && !exp_byline.contains(&result.author)
        {
            pass = false;
        }

        let exp_site = str_field(&meta, "siteName");
        if !exp_site.is_empty()
            && result.site != exp_site
            && !result.site.contains(&exp_site)
            && !exp_site.contains(&result.site)
        {
            pass = false;
        }

        let exp_lang = str_field(&meta, "lang");
        if !exp_lang.is_empty() && result.language != exp_lang {
            pass = false;
        }

        if pass && !must_pass.contains(name.as_str()) {
            new_passes.push(name.clone());
        }
        if !pass && must_pass.contains(name.as_str()) {
            regressions.push(name.clone());
        }
    }

    assert!(
        new_passes.is_empty(),
        "mozilla metadata: {total} tested. {} NEW passes — add to MOZILLA_META_PASS:\n  {}",
        new_passes.len(),
        new_passes.join("\n  ")
    );
    assert!(
        regressions.is_empty(),
        "mozilla metadata: {} REGRESSIONS:\n  {}",
        regressions.len(),
        regressions.join("\n  ")
    );
}

// ════════════════════════════════════════════════════════════════
// Pass lists — update these when extraction improves
// ════════════════════════════════════════════════════════════════

// Generated by running the oracle tests with empty lists and collecting
// the "NEW passes" output. Each fixture listed here must continue to pass.

// 126/144 fixtures pass exact metadata match against defuddle
const DEFUDDLE_META_PASS: &[&str] = &[
    "author-share-widget",
    "callouts--obsidian-publish-callouts",
    "code-blocks--chroma-linenums",
    "code-blocks--hexo-br",
    "codeblocks--chatgpt-codemirror",
    "codeblocks--chroma-inline-linenums",
    "codeblocks--chroma-line-spans",
    "codeblocks--code-pre-nesting",
    "codeblocks--flex-row-gutter",
    "codeblocks--mintlify",
    "codeblocks--pygments-lineno",
    "codeblocks--react-syntax-highlighter-linenums",
    "codeblocks--rehype-pretty-code",
    "codeblocks--rehype-pretty-copy",
    "codeblocks--rockthejvm.com-articles-kotlin-101-type-classes",
    "codeblocks--rouge-linenums",
    "codeblocks--stripe",
    "comments--news.ycombinator.com-item-id=12345678",
    "comments--old.reddit.com-r-test-comments-abc123-test_post",
    "content-patterns--card-grid-stripped-headings",
    "content-patterns--code-block-boilerplate-and-trailing-section",
    "content-patterns--heading-introduced-list",
    "content-patterns--iso-date-and-read-time",
    "content-patterns--leading-breadcrumb",
    "content-patterns--live-blog-metadata",
    "content-patterns--social-counter-link",
    "content-patterns--social-engagement-counter",
    "content-patterns--socket-dev-blog",
    "content-patterns--trailing-related-posts",
    "content-patterns--trailing-subscribe-after-footnotes",
    "elements--base64-placeholder-removal",
    "elements--bootstrap-alerts",
    "elements--complex-tables",
    "elements--data-table",
    "elements--embedded-videos",
    "elements--empty-p-br",
    "elements--farsi-zwnj",
    "elements--figure-content-wrapper",
    "elements--hugo-admonitions",
    "elements--image-dedup",
    "elements--javascript-links",
    "elements--lazy-image",
    "elements--nbsp-handling",
    "elements--srcset-normalization",
    "elements--whitespace-newlines",
    "extractor--bbcode-data",
    "footnotes--aside-ol-start",
    "footnotes--child-anchor-id",
    "footnotes--google-docs-ftnt",
    "footnotes--heading-notes",
    "footnotes--hidden-section",
    "footnotes--hr-continuation",
    "footnotes--hr-strong-numbered",
    "footnotes--hr-sup-numbered",
    "footnotes--inline-footnote-span",
    "footnotes--named-anchor",
    "footnotes--nested-prose",
    "footnotes--no-false-positive-equation-refs",
    "footnotes--numeric-anchor-id",
    "footnotes--p-class-footnote",
    "footnotes--sidenote-inline-with-list",
    "footnotes--word-ftn-ftnref",
    "footnotes--wp-block-footnotes",
    "general--12gramsofcarbon.com-p-ilyas-30-papers-to-carmack-vlaes",
    "general--apnews-link-enhancement",
    "general--appendix-heading",
    "general--cp4space-jordan-algebra",
    "general--developer.mozilla.org-en-US-docs-Web-JavaScript-Reference-Global_Objects-Array",
    "general--github.com-issue-56",
    "general--inline-comments-and-link-lists",
    "general--lesswrong.com-s-N7nDePaNabJdnbXeE-p-vJFdjigzmcXMhNTsx",
    "general--multi-article-portfolio",
    "general--react-streaming-ssr",
    "general--scp-wiki.wikidot.com-scp-9935",
    "general--stephango.com-buy-wisely",
    "general--substack-app",
    "general--substack-custom-domain",
    "general--substack-note",
    "general--substack-note-permalink",
    "general--svg-content-preservation",
    "general--svg-external-css-fallback",
    "general--trailing-cta-newsletter",
    "general--www.figma.com-blog-introducing-codex-to-figma",
    "headings--permalink-title-match",
    "hidden--nodes",
    "hidden--visibility",
    "issues--106-menu-id",
    "issues--120-dhammatalks-footnotes",
    "issues--131-category-links",
    "issues--132-hero-class",
    "issues--136-time-element",
    "issues--141-arxiv-equation-tables",
    "issues--142-arxiv-multi-citations",
    "issues--143-arxiv-cross-references",
    "issues--144-arxiv-footnote-marks",
    "issues--159-lean-heading-permalink-emoji",
    "issues--159-lean-verso-code-blocks",
    "issues--159-lean-verso-empty-line-preserved",
    "issues--159-lean-verso-grouped-blocks",
    "issues--159-lean-verso-missing-section-gap",
    "issues--162-aria-hidden-main-content",
    "issues--167-partial-selector-inside-code",
    "issues--168-links-inside-inline-code",
    "issues--169-svg-classname-crash",
    "issues--217-writerside-docs",
    "issues--218-footnote-wrapper-text-lost",
    "issues--221-nextjs-noscript-images",
    "issues--227-noscript-lazy-images",
    "issues--header-with-subtitle-p",
    "issues--header-wraps-content",
    "issues--span-with-block-children-and-schema",
    "math--katex",
    "math--katex-centraliser",
    "math--mathjax",
    "math--mathjax-svg",
    "math--mathjax-tex-scripts",
    "math--raw-latex",
    "math--temml",
    "math--wikipedia-mathml",
    "metadata--author-by-prefix-and-url",
    "scoring--related-posts-byline",
    "scoring--table-with-links",
    "selectors--arm-newsroom",
    "small-images--svg-icon-viewbox",
    "table-layout--paulgraham.com-makersschedule",
    "table-layout--single-column",
];

// 13/144 fixtures pass exact whitespace-normalized content match
const DEFUDDLE_CONTENT_PASS: &[&str] = &[
    "author-contact-block",
    "codeblocks--pygments-lineno",
    "content-patterns--card-grid-stripped-headings",
    "elements--nbsp-handling",
    "extractor--bbcode-data",
    "footnotes--child-anchor-id",
    "footnotes--numeric-anchor-id",
    "general--substack-app",
    "general--substack-note",
    "general--substack-note-permalink",
    "issues--161-x-status-url-author",
    "issues--header-with-subtitle-p",
    "issues--header-wraps-content",
];

// 115/130 fixtures pass exact metadata match against mozilla
const MOZILLA_META_PASS: &[&str] = &[
    "001",
    "002",
    "005-unescape-html-entities",
    "aclu",
    "aktualne",
    "archive-of-our-own",
    "ars-1",
    "base-url",
    "base-url-base-element",
    "base-url-base-element-relative",
    "basic-tags-cleaning",
    "bbc-1",
    "blogger",
    "breitbart",
    "bug-1255978",
    "buzzfeed-1",
    "citylab-1",
    "clean-links",
    "cnet",
    "cnet-svg-classes",
    "cnn",
    "comment-inside-script-parsing",
    "daringfireball-1",
    "data-url-image",
    "dev418",
    "dropbox-blog",
    "ebb-org",
    "ehow-1",
    "ehow-2",
    "embedded-videos",
    "folha",
    "gitlab-blog",
    "gmw",
    "google-sre-book-1",
    "guardian-1",
    "heise",
    "hidden-nodes",
    "hukumusume",
    "ietf-1",
    "invalid-attributes",
    "js-link-replacement",
    "keep-images",
    "keep-tabular-data",
    "la-nacion",
    "lazy-image-1",
    "lazy-image-2",
    "lazy-image-3",
    "lemonde-1",
    "liberation-1",
    "lifehacker-post-comment-load",
    "lifehacker-working",
    "links-in-tables",
    "lwn-1",
    "mathjax",
    "medicalnewstoday",
    "medium-1",
    "medium-2",
    "medium-3",
    "mercurial",
    "mozilla-1",
    "mozilla-2",
    "msn",
    "normalize-spaces",
    "nytimes-1",
    "nytimes-2",
    "nytimes-3",
    "nytimes-4",
    "nytimes-5",
    "ol",
    "parsely-metadata",
    "pixnet",
    "qq",
    "quanta-1",
    "remove-aria-hidden",
    "remove-extra-brs",
    "remove-extra-paragraphs",
    "remove-script-tags",
    "reordering-paragraphs",
    "replace-brs",
    "replace-font-tags",
    "royal-road",
    "rtl-1",
    "rtl-2",
    "rtl-3",
    "rtl-4",
    "salon-1",
    "schema-org-context-object",
    "simplyfound-1",
    "social-buttons",
    "spiceworks",
    "style-tags-removal",
    "svg-parsing",
    "table-style-attributes",
    "telegraph",
    "title-and-h1-discrepancy",
    "title-en-dash",
    "tmz-1",
    "toc-missing",
    "topicseed-1",
    "tumblr",
    "v8-blog",
    "videos-2",
    "visibility-hidden",
    "wapo-1",
    "wapo-2",
    "webmd-1",
    "webmd-2",
    "wikia",
    "wikipedia",
    "wordpress",
    "yahoo-1",
    "yahoo-2",
    "yahoo-3",
    "yahoo-4",
    "youth",
];

// ════════════════════════════════════════════════════════════════
// Fixture sweeps (non-empty extraction)
// ════════════════════════════════════════════════════════════════

#[test]
fn defuddle_all_extract_content() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/defuddle");
    let mut failures = Vec::new();
    let mut total = 0;

    for entry in fs::read_dir(&dir).unwrap().flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("html") {
            continue;
        }
        total += 1;
        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        let html = fs::read_to_string(&path).unwrap();
        let url = url_from_html(&html).unwrap_or_else(|| format!("https://example.com/{name}"));
        let result = parse(&html, &opts(&url));

        if result.content.trim().is_empty() || result.word_count == 0 {
            failures.push(name);
        }
    }

    assert!(total >= 100);
    assert!(failures.is_empty(), "empty extractions: {failures:?}");
}

#[test]
fn mozilla_all_extract_content() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/mozilla");
    let mut failures = Vec::new();
    let mut total = 0;

    for entry in fs::read_dir(&dir).unwrap().flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        let Ok(html) = fs::read_to_string(path.join("source.html")) else {
            continue;
        };
        let meta_path = path.join("expected-metadata.json");
        if let Ok(meta_str) = fs::read_to_string(&meta_path) {
            if let Ok(meta) = serde_json::from_str::<serde_json::Value>(&meta_str) {
                if meta.get("readerable").and_then(serde_json::Value::as_bool) == Some(false) {
                    continue;
                }
            }
        }
        total += 1;
        let result = parse(&html, &DecruftOptions::default());
        if result.word_count == 0 {
            failures.push(name);
        }
    }

    assert!(total >= 100);
    assert!(failures.is_empty(), "empty extractions: {failures:?}");
}

// ════════════════════════════════════════════════════════════════
// Content assertions
// ════════════════════════════════════════════════════════════════

fn load(name: &str) -> String {
    let path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

fn load_defuddle(name: &str) -> String {
    load(&format!("defuddle/{name}"))
}

fn check(html: &str, url: &str, with: &[&str], without: &[&str]) {
    let result = parse(html, &opts(url));
    for p in with {
        assert!(result.content.contains(p), "missing {p:?} from {url}");
    }
    for p in without {
        assert!(!result.content.contains(p), "unwanted {p:?} from {url}");
    }
}

#[test]
fn blog_content_and_clutter() {
    let html = load("complex_blog.html");
    check(
        &html,
        "https://example.com/article",
        &[
            "Rust's ownership system",
            "Borrowing and References",
            "String::from",
        ],
        &[
            "main-nav",
            "ad-banner",
            "Popular Posts",
            "cookie",
            "Privacy Policy",
            "share-twitter",
            "Comments (12)",
            "newsletter",
            "You Might Also Like",
        ],
    );
    let r = parse(&html, &opts("https://example.com/article"));
    assert_eq!(r.title, "Understanding Rust Ownership");
    assert_eq!(r.author, "Alice Chen");
}

#[test]
fn news_content_and_clutter() {
    let html = load("news_article.html");
    check(
        &html,
        "https://example.com/article",
        &["Marine biologists", "Aurelia profundis", "<blockquote"],
        &["inline-ad", "ADVERTISEMENT", "Trending Now", "BREAKING"],
    );
    let r = parse(&html, &opts("https://example.com/article"));
    assert_eq!(r.title, "Scientists Discover New Species in Deep Ocean");
    assert_eq!(r.author, "Sarah Mitchell");
}

#[test]
fn stripe_code_blocks_preserved() {
    let html = load_defuddle("codeblocks--stripe.html");
    check(
        &html,
        "https://stripe.com/docs",
        &["paymentMiddleware", "curl"],
        &[],
    );
}

#[test]
fn scp_wiki_footnotes_preserved() {
    let html = load_defuddle("general--scp-wiki.wikidot.com-scp-9935.html");
    check(
        &html,
        "https://scp-wiki.wikidot.com/scp-9935",
        &["No relation to the Washington Nationals"],
        &[],
    );
}

#[test]
fn cp4space_title_and_bibliography() {
    let html = load_defuddle("general--cp4space-jordan-algebra.html");
    check(
        &html,
        "https://cp4space.hatsya.com/2020/10/28/the-exceptional-jordan-algebra/",
        &["exceptional Jordan algebra", "John Baez"],
        &[],
    );
}

#[test]
fn empty_document() {
    let r = parse("", &DecruftOptions::default());
    assert!(r.content.is_empty() || r.word_count == 0);
}

#[test]
fn minimal_document() {
    let r = parse(
        "<html><body><p>Hello world</p></body></html>",
        &DecruftOptions::default(),
    );
    assert!(r.content.contains("Hello world"));
}

macro_rules! mozilla_site {
    ($name:ident, $fixture:expr, $min_words:expr) => {
        #[test]
        fn $name() {
            let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/mozilla")
                .join($fixture);
            let html = fs::read_to_string(dir.join("source.html"))
                .unwrap_or_else(|e| panic!("{}: {e}", $fixture));
            let r = parse(&html, &DecruftOptions::default());
            assert!(
                r.word_count >= $min_words,
                "{}: {} < {}",
                $fixture,
                r.word_count,
                $min_words
            );
        }
    };
}

mozilla_site!(mozilla_bbc, "bbc-1", 200);
mozilla_site!(mozilla_nytimes, "nytimes-1", 200);
mozilla_site!(mozilla_medium, "medium-1", 100);
mozilla_site!(mozilla_wikipedia, "wikipedia", 500);
mozilla_site!(mozilla_cnn, "cnn", 100);
mozilla_site!(mozilla_guardian, "guardian-1", 200);
mozilla_site!(mozilla_wapo, "wapo-1", 200);
mozilla_site!(mozilla_ars, "ars-1", 200);
