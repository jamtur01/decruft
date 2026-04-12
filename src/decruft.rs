use std::time::Instant;

use ego_tree::NodeId;
use scraper::{Html, Node};

use crate::callouts;
use crate::cleanup;
use crate::code_blocks;
use crate::content;
use crate::dom;
use crate::extractors;
use crate::footnotes;
use crate::math;
use crate::metadata;
use crate::metadata_block;
use crate::noscript;
use crate::patterns;
use crate::schema_org;
use crate::standardize;
use crate::streaming_ssr;
use crate::types::{DebugInfo, DecruftOptions, DecruftResult, Removal};

/// Result of a single parse-internal pass.
struct ParseResult {
    content: String,
    word_count: usize,
    content_selector_path: String,
    removals: Vec<Removal>,
}

/// Parse HTML and extract clean, readable content.
///
/// Runs the full extraction pipeline: metadata extraction, site-specific
/// extractors, content scoring, and multi-stage cleanup with progressive
/// relaxation when too little content is found.
///
/// # Arguments
///
/// * `html_str` - Raw HTML string to parse.
/// * `options` - Configuration for the extraction pipeline. Use
///   [`DecruftOptions::default()`] for sensible defaults.
///
/// # Returns
///
/// A [`DecruftResult`] containing the cleaned content, metadata, and
/// optionally Markdown and debug information.
///
/// # Examples
///
/// ```
/// use decruft::{parse, DecruftOptions};
///
/// let html = "<html><body><article><p>Hello world</p></article></body></html>";
/// let result = parse(html, &DecruftOptions::default());
/// assert!(result.content.contains("Hello world"));
/// ```
#[must_use]
pub fn parse(html_str: &str, options: &DecruftOptions) -> DecruftResult {
    let start = Instant::now();
    let sanitized = sanitize_html_comments(html_str);
    let repaired = repair_surrogate_pairs(&sanitized);
    let html_str = repaired.as_str();

    let html = Html::parse_document(html_str);
    let schema_data = schema_org::extract_schema_org(&html);
    let meta_tags = cleanup::collect_meta_tags(&html);
    let mut meta = metadata::extract_metadata(&html, options.url.as_deref(), schema_data.as_ref());

    // Try specialized extractors before the general pipeline
    if let Some(result) = try_extractors(
        &html,
        &start,
        options,
        &mut meta,
        schema_data.as_ref(),
        &meta_tags,
    ) {
        return result;
    }

    // Stage 0: default pipeline
    let mut result = parse_internal(html_str, options);

    // User-specified content selector is a hard override — no retries.
    // The retry stages can replace the selector with body, hidden
    // subtree, or schema selector, undermining the user's choice.
    if options.content_selector.is_none() {
        // ── Retry strategy ─────────────────────────────────────────
        // Each stage relaxes cleanup constraints progressively when
        // the extracted content is too short, measured by word count:
        //
        //   Stage 1 (<200 wc):  drop partial-selector removal
        //   Stage 2 (<50 wc):   drop hidden-element removal; also try
        //                        targeting the largest hidden subtree
        //   Stage 3 (<30 wc):   drop scoring and content patterns;
        //                        try body as fallback content root
        //   Stage 4 (always):   if schema.org text has 1.5x more
        //                        words, locate the DOM element or use
        //                        raw text
        // ──────────────────────────────────────────────────────────

        // Stage 1: relax partial selectors
        if result.word_count < 200 {
            result = retry_without_partials(html_str, options, result);
        }

        // Stage 2: relax hidden-element removal
        if result.word_count < 50 {
            result = retry_without_hidden(html_str, options, result);
        }

        // Stage 3: fully relaxed (no scoring, no patterns)
        if result.word_count < 30 {
            result = retry_fully_relaxed(html_str, options, result);
        }

        // Stage 4: schema.org articleBody fallback
        result = retry_schema_fallback(html_str, options, schema_data.as_ref(), result);
    }

    let elapsed = start.elapsed();
    #[expect(clippy::cast_possible_truncation)]
    let parse_time_ms = elapsed.as_millis() as u64;

    let content_markdown = if options.markdown || options.separate_markdown {
        convert_to_markdown(&result.content)
    } else {
        None
    };

    let content = if options.markdown {
        content_markdown.clone().unwrap_or(result.content)
    } else {
        result.content
    };

    build_result(
        content,
        content_markdown,
        result.word_count,
        parse_time_ms,
        meta,
        schema_data,
        meta_tags,
        result.content_selector_path,
        result.removals,
        options.debug,
    )
}

/// Run the full extraction pipeline once with the given options.
/// Each call re-parses from raw HTML, applies noscript resolution,
/// then runs the cleanup pipeline.
fn parse_internal(html_str: &str, options: &DecruftOptions) -> ParseResult {
    let mut html = Html::parse_document(html_str);
    let mut removals: Vec<Removal> = Vec::new();

    // Pre-processing
    noscript::resolve_noscript_images(&mut html);
    streaming_ssr::resolve_streaming_ssr(&mut html);
    standardize::strip_unsafe_elements(&mut html);

    let main_content = resolve_content_root(&html, options);
    let content_selector_path = dom::selector_path(&html, main_content);

    run_cleanup_pipeline(&mut html, main_content, &mut removals, options);

    let content = dom::outer_html(&html, main_content);
    let word_count = dom::count_words_html(&content);

    ParseResult {
        content,
        word_count,
        content_selector_path,
        removals,
    }
}

/// RETRY 1: If result has < 200 words, retry without partial selectors.
/// Only use the retry if it yields a significant improvement. For
/// short pages (< 50 words), require 5x improvement to avoid
/// overriding correct cleanup with junk content.
fn retry_without_partials(
    html_str: &str,
    options: &DecruftOptions,
    current: ParseResult,
) -> ParseResult {
    let mut opts = options.clone();
    opts.remove_partial_selectors = false;
    let retry = parse_internal(html_str, &opts);
    let threshold = if current.word_count < 50 {
        current.word_count * 5
    } else {
        current.word_count * 2
    };
    if retry.word_count > threshold {
        retry
    } else {
        current
    }
}

/// RETRY 2: If result has < 50 words, retry without hidden element
/// removal, then try targeting the largest hidden content subtree.
fn retry_without_hidden(
    html_str: &str,
    options: &DecruftOptions,
    mut result: ParseResult,
) -> ParseResult {
    // RETRY 2a: no hidden element removal
    let mut opts = options.clone();
    opts.remove_hidden_elements = false;
    let retry = parse_internal(html_str, &opts);
    if retry.word_count > result.word_count * 2 {
        result = retry;
    }

    // RETRY 2b: target largest hidden content subtree
    if let Some(selector) = find_largest_hidden_content_selector(html_str) {
        let mut opts2 = options.clone();
        opts2.remove_hidden_elements = false;
        opts2.remove_partial_selectors = false;
        opts2.content_selector = Some(selector);
        let retry2 = parse_internal(html_str, &opts2);
        if should_prefer_hidden_retry(&retry2, &result) {
            result = retry2;
        }
    }

    result
}

/// Check if the hidden-subtree retry is better than the current result.
fn should_prefer_hidden_retry(retry: &ParseResult, current: &ParseResult) -> bool {
    if retry.word_count > current.word_count {
        return true;
    }
    let threshold = std::cmp::max(20, current.word_count * 7 / 10);
    retry.word_count > threshold && retry.content.len() < current.content.len()
}

/// RETRY 3: Disable scoring, partials, and content patterns.
/// Also try body as explicit content selector if still low.
fn retry_fully_relaxed(
    html_str: &str,
    options: &DecruftOptions,
    result: ParseResult,
) -> ParseResult {
    let mut opts = options.clone();
    opts.remove_low_scoring = false;
    opts.remove_partial_selectors = false;
    opts.remove_content_patterns = false;
    opts.remove_hidden_elements = false;
    let mut best = parse_internal(html_str, &opts);
    if best.word_count <= result.word_count {
        best = result;
    }

    // If still low, try body as explicit content selector
    if best.word_count < 50 {
        let mut body_opts = opts;
        body_opts.content_selector = Some("body".to_string());
        let body_retry = parse_internal(html_str, &body_opts);
        if body_retry.word_count > best.word_count {
            return body_retry;
        }
    }

    best
}

/// RETRY 4: Use schema.org text as fallback content.
fn retry_schema_fallback(
    html_str: &str,
    options: &DecruftOptions,
    schema_data: Option<&serde_json::Value>,
    mut result: ParseResult,
) -> ParseResult {
    let Some(data) = schema_data else {
        return result;
    };
    let Some(schema_text) = schema_org::get_text(data) else {
        return result;
    };
    let schema_wc = dom::count_words_html(&schema_text);
    if schema_wc <= result.word_count * 3 / 2 {
        return result;
    }

    if let Some(selector) = find_element_by_schema_text(html_str, &schema_text) {
        let mut opts = options.clone();
        opts.content_selector = Some(selector);
        result = parse_internal(html_str, &opts);
    } else {
        result.content = standardize::sanitize_html_string(&schema_text);
        result.word_count = schema_wc;
        result.content_selector_path = "schema.org/articleBody".to_string();
        result.removals.clear();
    }
    result
}

/// Find the CSS selector for the largest hidden element with
/// substantial content (>= 30 words).
///
/// Uses the centralized `is_hidden_element` check, which covers
/// inline styles, `[hidden]`, `[aria-hidden="true"]`, `[inert]`,
/// `.hidden`, `:hidden`, and `.invisible`.
fn find_largest_hidden_content_selector(html_str: &str) -> Option<String> {
    let html = Html::parse_document(html_str);

    let mut best_selector: Option<String> = None;
    let mut best_word_count: usize = 0;

    for id in dom::select_ids(&html, "*") {
        if !cleanup::is_hidden_element(&html, id) {
            continue;
        }
        let text = dom::text_content(&html, id);
        let wc = dom::count_words(&text);
        if wc >= 30 && wc > best_word_count {
            best_word_count = wc;
            best_selector = Some(build_unique_selector(&html, id));
        }
    }

    best_selector
}

/// Build a CSS selector that uniquely identifies a node.
/// Prefers #id, falls back to the full selector path.
fn build_unique_selector(html: &Html, node_id: NodeId) -> String {
    let Some(node_ref) = html.tree.get(node_id) else {
        return String::new();
    };
    let Node::Element(el) = node_ref.value() else {
        return String::new();
    };

    if let Some(id_attr) = el.attr("id") {
        let sel = format!("#{id_attr}");
        if dom::select_ids(html, &sel).len() == 1 {
            return sel;
        }
    }

    dom::selector_path(html, node_id)
}

/// Find the smallest DOM element containing the first 100 chars of
/// the schema text's first paragraph and >= 80% of the schema word
/// count.
fn find_element_by_schema_text(html_str: &str, schema_text: &str) -> Option<String> {
    let prefix = extract_first_paragraph_prefix(schema_text, 100);
    if prefix.is_empty() {
        return None;
    }

    let html = Html::parse_document(html_str);
    let schema_wc = dom::count_words_html(schema_text);
    let threshold = schema_wc * 4 / 5; // 80%

    let mut best_id: Option<NodeId> = None;
    let mut best_text_len = usize::MAX;

    for id in dom::select_ids(&html, "*") {
        let text = dom::text_content(&html, id);
        if !text.contains(&prefix) {
            continue;
        }
        let wc = dom::count_words(&text);
        if wc < threshold {
            continue;
        }
        if text.len() < best_text_len {
            best_text_len = text.len();
            best_id = Some(id);
        }
    }

    best_id.map(|id| build_unique_selector(&html, id))
}

/// Extract the first `max_chars` characters from the first paragraph
/// of text (after stripping HTML tags).
fn extract_first_paragraph_prefix(html_text: &str, max_chars: usize) -> String {
    let plain = dom::strip_html_tags(html_text);
    let trimmed = plain.trim();

    let paragraph = trimmed.split("\n\n").next().unwrap_or(trimmed).trim();

    safe_truncate(paragraph, max_chars).to_string()
}

/// Truncate a string to at most `max_chars` characters, respecting
/// UTF-8 char boundaries.
fn safe_truncate(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        Some((i, _)) => &s[..i],
        None => s,
    }
}

/// Repair HTML numeric character references for surrogate pairs.
///
/// Draft.js (used by X/Twitter articles) splits emoji at the JS
/// surrogate boundary, producing high/low surrogate char refs that
/// may be separated by HTML tags: `&#55357;<span>&#56580;`.
/// html5ever replaces lone surrogates with U+FFFD, destroying the
/// emoji.
///
/// This function finds high surrogate char refs followed by a low
/// surrogate char ref (possibly separated by HTML tags) and merges
/// them into the correct Unicode character.
fn repair_surrogate_pairs(html: &str) -> String {
    use regex::Regex;
    use std::sync::LazyLock;

    // Match a high surrogate char ref, optionally followed by HTML
    // tags (no text content between them), then a low surrogate ref.
    // The tags between surrogates are captured as a single group
    // using a greedy match of tag sequences.
    static SURROGATE_PAIR_RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(
            r"(?i)&#(?:x([dD][89aAbB][0-9a-fA-F]{2})|(5[5-6]\d{3}));((?:<[^>]*>)*)&#(?:x([dD][cCdDeEfF][0-9a-fA-F]{2})|(5[6-7]\d{3}));"
        )
        .expect("surrogate pair regex is valid")
    });

    SURROGATE_PAIR_RE
        .replace_all(html, |caps: &regex::Captures| {
            let high = parse_surrogate_cap(caps, 1, 2);
            let low = parse_surrogate_cap(caps, 4, 5);
            match (high, low) {
                (Some(h), Some(l))
                    if (0xD800..=0xDBFF).contains(&h) && (0xDC00..=0xDFFF).contains(&l) =>
                {
                    let code_point = ((h - 0xD800) << 10) + (l - 0xDC00) + 0x10000;
                    let tags = caps.get(3).map_or("", |m| m.as_str());
                    char::from_u32(code_point)
                        .map_or_else(|| caps[0].to_string(), |c| format!("{tags}{c}"))
                }
                _ => caps[0].to_string(),
            }
        })
        .into_owned()
}

/// Parse a surrogate value from a regex capture: group `hex_idx` is
/// the hex digits, group `dec_idx` is the decimal digits.
fn parse_surrogate_cap(caps: &regex::Captures, hex_idx: usize, dec_idx: usize) -> Option<u32> {
    if let Some(hex) = caps.get(hex_idx) {
        u32::from_str_radix(hex.as_str(), 16).ok()
    } else if let Some(dec) = caps.get(dec_idx) {
        dec.as_str().parse::<u32>().ok()
    } else {
        None
    }
}

/// Strip HTML-like tags from inside comments to prevent html5ever
/// from mis-parsing the DOM tree. Comments containing `<p>`, `<br>`,
/// etc. can break sibling element detection.
fn sanitize_html_comments(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut remaining = html;

    while let Some(start) = remaining.find("<!--") {
        result.push_str(&remaining[..start]);
        let after_open = &remaining[start + 4..];
        if let Some(end) = after_open.find("-->") {
            result.push_str("<!--");
            let comment_body = &after_open[..end];
            // Replace < and > inside the comment with safe chars
            for ch in comment_body.chars() {
                match ch {
                    '<' | '>' => result.push('\u{FFFD}'),
                    _ => result.push(ch),
                }
            }
            result.push_str("-->");
            remaining = &after_open[end + 3..];
        } else {
            // Unclosed comment: push rest as-is
            result.push_str(&remaining[start..]);
            remaining = "";
        }
    }
    result.push_str(remaining);
    result
}

fn resolve_content_root(html: &Html, options: &DecruftOptions) -> NodeId {
    if let Some(ref sel) = options.content_selector {
        dom::select_ids(html, sel)
            .into_iter()
            .next()
            .unwrap_or_else(|| find_main(html))
    } else {
        find_main(html)
    }
}

fn run_cleanup_pipeline(
    html: &mut Html,
    main_content: NodeId,
    removals: &mut Vec<Removal>,
    options: &DecruftOptions,
) {
    // Standardize math elements early, before selectors might remove
    // them (e.g. MathJax classes matching partial removal patterns).
    if options.standardize {
        math::standardize_math(html, main_content);
    }
    footnotes::standardize_footnotes(html, main_content);
    callouts::standardize_callouts(html, main_content);

    if options.remove_images {
        cleanup::remove_all_images(html, main_content);
    }
    if options.remove_small_images {
        cleanup::remove_small_images(html, main_content);
    }
    if options.remove_exact_selectors {
        cleanup::remove_exact_selectors(html, main_content, removals, options.debug);
        cleanup::remove_header_elements(html, main_content, removals, options.debug);
    }
    cleanup::run_combined_cleanup(
        html,
        main_content,
        removals,
        options.debug,
        options.remove_hidden_elements,
        options.remove_partial_selectors,
        options.remove_low_scoring,
    );
    metadata_block::remove_metadata_block(html, main_content);
    if options.remove_content_patterns {
        patterns::remove_content_patterns(html, main_content, removals, options.debug);
    }
    if options.standardize {
        code_blocks::standardize_code_blocks(html, main_content);
        standardize::standardize_content(html, main_content);
    }
    if let Some(ref url) = options.url {
        standardize::resolve_urls(html, main_content, url);
    }
    cleanup::deduplicate_images(html, main_content);
}

#[expect(clippy::too_many_arguments)]
fn build_result(
    content: String,
    content_markdown: Option<String>,
    word_count: usize,
    parse_time_ms: u64,
    meta: crate::types::Metadata,
    schema_data: Option<serde_json::Value>,
    meta_tags: Vec<crate::types::MetaTag>,
    content_selector_path: String,
    removals: Vec<Removal>,
    debug: bool,
) -> DecruftResult {
    DecruftResult {
        content,
        content_markdown,
        title: meta.title,
        description: meta.description,
        domain: meta.domain,
        favicon: meta.favicon,
        image: meta.image,
        language: meta.language,
        parse_time_ms,
        published: meta.published,
        modified: meta.modified,
        author: meta.author,
        site: meta.site_name,
        canonical_url: meta.canonical_url,
        keywords: meta.keywords,
        content_type: meta.content_type,
        word_count,
        schema_org_data: schema_data,
        extractor_type: None,
        meta_tags: if debug { Some(meta_tags) } else { None },
        debug: if debug {
            Some(DebugInfo {
                content_selector: content_selector_path,
                removals,
            })
        } else {
            None
        },
    }
}

fn find_main(html: &Html) -> NodeId {
    content::find_main_content(html)
}

/// Sanitize extractor-produced HTML through the same pipeline as
/// the general extraction path: strip unsafe elements, resolve
/// relative URLs, and clean non-allowed attributes.
fn sanitize_extractor_content(html_str: &str, url: Option<&str>) -> String {
    let mut html = Html::parse_fragment(html_str);
    standardize::strip_unsafe_elements(&mut html);
    let root = html.tree.root().id();
    if let Some(base) = url {
        standardize::resolve_urls(&mut html, root, base);
    }
    standardize::clean_attributes_on(&mut html, root);
    dom::inner_html(&html, root)
}

/// Try specialized extractors (`BBCode`, Substack) before the general
/// pipeline. Returns `Some` with the result if an extractor matched.
fn try_extractors(
    html: &Html,
    start: &Instant,
    options: &DecruftOptions,
    meta: &mut crate::types::Metadata,
    schema_data: Option<&serde_json::Value>,
    meta_tags: &[crate::types::MetaTag],
) -> Option<DecruftResult> {
    if let Some(result) = try_bbcode(html, start, options, meta, schema_data, meta_tags) {
        return Some(result);
    }
    if let Some(result) = try_substack(html, start, options, meta, schema_data, meta_tags) {
        return Some(result);
    }
    try_site_extractors(html, start, options, meta, schema_data, meta_tags)
}

fn try_bbcode(
    html: &Html,
    start: &Instant,
    options: &DecruftOptions,
    meta: &mut crate::types::Metadata,
    schema_data: Option<&serde_json::Value>,
    meta_tags: &[crate::types::MetaTag],
) -> Option<DecruftResult> {
    let bbcode = extractors::bbcode::extract_bbcode_content(html)?;
    if let Some(t) = &bbcode.title {
        if let Some(v) = non_empty(t) {
            meta.title = Some(v);
        }
    }
    if let Some(a) = &bbcode.author {
        if let Some(v) = non_empty(a) {
            meta.author = Some(v);
        }
    }
    let word_count = dom::count_words_html(&bbcode.html);
    if word_count == 0 {
        return None;
    }
    let mut result = build_extractor_result(
        &bbcode.html,
        "div[data-partnereventstore]",
        start,
        options,
        meta,
        schema_data,
        meta_tags,
        word_count,
    );
    result.extractor_type = Some("bbcode".to_string());
    Some(result)
}

fn try_substack(
    html: &Html,
    start: &Instant,
    options: &DecruftOptions,
    meta: &mut crate::types::Metadata,
    schema_data: Option<&serde_json::Value>,
    meta_tags: &[crate::types::MetaTag],
) -> Option<DecruftResult> {
    let substack = extractors::substack::extract_substack_content(html, options.url.as_deref())?;
    apply_substack_meta(&substack, meta);
    let word_count = dom::count_words_html(&substack.html);
    if word_count == 0 {
        return None;
    }
    let mut result = build_extractor_result(
        &substack.html,
        "feedCommentBody",
        start,
        options,
        meta,
        schema_data,
        meta_tags,
        word_count,
    );
    result.extractor_type = Some("substack".to_string());
    Some(result)
}

#[expect(clippy::too_many_arguments)]
fn build_extractor_result(
    raw_html: &str,
    selector_label: &str,
    start: &Instant,
    options: &DecruftOptions,
    meta: &crate::types::Metadata,
    schema_data: Option<&serde_json::Value>,
    meta_tags: &[crate::types::MetaTag],
    word_count: usize,
) -> DecruftResult {
    let sanitized = sanitize_extractor_content(raw_html, options.url.as_deref());
    let content_markdown = if options.markdown || options.separate_markdown {
        convert_to_markdown(&sanitized)
    } else {
        None
    };
    let content = if options.markdown {
        content_markdown
            .clone()
            .unwrap_or_else(|| sanitized.clone())
    } else {
        sanitized
    };
    let elapsed = start.elapsed();
    #[expect(clippy::cast_possible_truncation)]
    let parse_time_ms = elapsed.as_millis() as u64;
    build_result(
        content,
        content_markdown,
        word_count,
        parse_time_ms,
        meta.clone(),
        schema_data.cloned(),
        meta_tags.to_vec(),
        selector_label.to_string(),
        Vec::new(),
        options.debug,
    )
}

/// Try site-specific extractors (GitHub, Reddit, Hacker News, etc.).
fn try_site_extractors(
    html: &Html,
    start: &Instant,
    options: &DecruftOptions,
    meta: &mut crate::types::Metadata,
    schema_data: Option<&serde_json::Value>,
    meta_tags: &[crate::types::MetaTag],
) -> Option<DecruftResult> {
    let (extracted, extractor_name) =
        extractors::try_extract(html, options.url.as_deref(), options.include_replies)?;
    apply_extractor_metadata(&extracted, meta);
    let word_count = dom::count_words_html(&extracted.content);
    if word_count == 0 {
        return None;
    }
    let mut result = build_extractor_result(
        &extracted.content,
        "site-extractor",
        start,
        options,
        meta,
        schema_data,
        meta_tags,
        word_count,
    );
    result.extractor_type = Some(extractor_name.to_string());
    Some(result)
}

/// Normalize a string to `None` if empty or whitespace-only.
fn non_empty(s: &str) -> Option<String> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// Apply site-specific extractor metadata onto the pipeline metadata.
///
/// Title, author, and site always override (extractors are more
/// reliable for these). Published, image, and description only fill
/// in when the pipeline left them as `None`.
fn apply_extractor_metadata(
    extracted: &extractors::ExtractorResult,
    meta: &mut crate::types::Metadata,
) {
    if let Some(t) = &extracted.title {
        let cleaned = metadata::clean_title(t, "", None, None);
        if let Some(v) = non_empty(&cleaned) {
            meta.title = Some(v);
        }
    }
    // Extractor author is always preferred -- site-specific
    // extractors identify the post author more reliably than the
    // generic metadata pipeline (which may pick up commenter names).
    if let Some(a) = &extracted.author {
        if let Some(v) = non_empty(a) {
            meta.author = Some(v);
        }
    }
    if let Some(s) = &extracted.site {
        if let Some(v) = non_empty(s) {
            meta.site_name = Some(v);
        }
    }
    if let Some(p) = &extracted.published
        && meta.published.is_none()
    {
        meta.published = non_empty(p);
    }
    if let Some(img) = &extracted.image
        && meta.image.is_none()
    {
        meta.image = non_empty(img);
    }
    if let Some(d) = &extracted.description
        && meta.description.is_none()
    {
        meta.description = non_empty(d);
    }
}

/// Apply Substack-extracted metadata to the result metadata.
fn apply_substack_meta(
    substack: &extractors::substack::SubstackContent,
    meta: &mut crate::types::Metadata,
) {
    if let Some(t) = &substack.title
        && meta.title.is_none()
    {
        meta.title = non_empty(t);
    }
    if let Some(a) = &substack.author
        && meta.author.is_none()
    {
        meta.author = non_empty(a);
    }
    if let Some(s) = &substack.site
        && meta.site_name.is_none()
    {
        meta.site_name = non_empty(s);
    }
    if let Some(img) = &substack.image
        && meta.image.is_none()
    {
        meta.image = non_empty(img);
    }
}

/// Convert HTML to Markdown with custom handlers for math
/// (`data-latex`), footnotes, and embedded media.
fn convert_to_markdown(html: &str) -> Option<String> {
    use htmd::HtmlToMarkdownBuilder;
    use htmd::element_handler::HandlerResult;
    use htmd::element_handler::Handlers;

    HtmlToMarkdownBuilder::new()
        .add_handler(
            vec!["iframe"],
            |_handlers: &dyn Handlers, element: htmd::Element| {
                let src = element
                    .attrs
                    .iter()
                    .find(|a| a.name.local.as_ref() == "src")
                    .map(|a| a.value.to_string())?;
                let url = if src.contains("youtube.com/embed/") {
                    src.replace("/embed/", "/watch?v=")
                } else {
                    src
                };
                Some(HandlerResult::from(format!("\n![]({url})")))
            },
        )
        .add_handler(vec!["sup"], handle_sup_element)
        .add_handler(vec!["span", "div"], handle_span_div_element)
        .add_handler(vec!["a"], handle_anchor_element)
        .build()
        .convert(html)
        .ok()
        .map(|s| fix_bang_image_collision(s.as_str()))
        .map(|s| unescape_latex_delimiters(&s))
        .map(|s| clean_bare_bullets(&s))
        .map(|s| collapse_newlines(&s))
}

/// Collapse runs of 3+ newlines into exactly 2 (one blank line),
/// but only outside fenced code blocks where multiple blank lines
/// may be semantically meaningful.
fn collapse_newlines(md: &str) -> String {
    let mut out = Vec::new();
    let lines: Vec<&str> = md.split('\n').collect();
    let mut in_fence = false;
    let mut blank_run = 0;

    for line in &lines {
        let trimmed = line.trim();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
        }

        if line.is_empty() {
            blank_run += 1;
            if in_fence || blank_run <= 1 {
                out.push(*line);
            }
        } else {
            blank_run = 0;
            out.push(*line);
        }
    }
    out.join("\n")
}

/// Remove bare bullet lines (a lone `-`, `+`, or `*` with no text)
/// that htmd produces for certain empty or nested-only list items.
fn clean_bare_bullets(md: &str) -> String {
    let mut out = Vec::new();
    let lines: Vec<&str> = md.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed == "-" || trimmed == "+" || trimmed == "*" {
            let next = lines.get(i + 1).map(|s| s.trim());
            // Remove if followed by empty line, another bare bullet, or end of input
            if next.is_none()
                || next == Some("")
                || next == Some("-")
                || next == Some("+")
                || next == Some("*")
            {
                continue;
            }
        }
        out.push(*line);
    }
    out.join("\n")
}

/// Handle `<sup>` elements: convert canonical footnote refs to `[^N]`.
#[expect(clippy::needless_pass_by_value, clippy::unnecessary_wraps)]
fn handle_sup_element(
    handlers: &dyn htmd::element_handler::Handlers,
    element: htmd::Element,
) -> Option<htmd::element_handler::HandlerResult> {
    let attrs = collect_attrs(&element);
    if let Some(num) = footnotes::is_canonical_footnote_ref(&attrs) {
        // Strip any sub-ref suffix (e.g. "1-2" -> "1")
        let base = num.split('-').next().unwrap_or(&num);
        return Some(htmd::element_handler::HandlerResult::from(format!(
            "[^{base}]"
        )));
    }
    // For non-footnote sups, render as <sub>-style or just inline
    Some(handlers.walk_children(element.node))
}

/// Handle `<span>` and `<div>` elements: data-latex math or
/// canonical footnotes container.
fn handle_span_div_element(
    handlers: &dyn htmd::element_handler::Handlers,
    element: htmd::Element,
) -> Option<htmd::element_handler::HandlerResult> {
    // Check for data-latex math
    let latex = element
        .attrs
        .iter()
        .find(|a| a.name.local.as_ref() == "data-latex")
        .map(|a| a.value.to_string());
    if let Some(l) = latex
        && !l.is_empty()
    {
        let is_block = element.tag == "div";
        let md = if is_block {
            format!("\n$$\n{l}\n$$\n")
        } else {
            format!("${l}$")
        };
        return Some(htmd::element_handler::HandlerResult::from(md));
    }

    // Check for canonical footnotes div
    if element.tag == "div" {
        let attrs = collect_attrs(&element);

        // Footnote item: div#fn:N.footnote -> [^N]: content
        if let Some(num) = footnotes::is_canonical_footnote_item(&attrs) {
            let base = num.split('-').next().unwrap_or(&num);
            let content = handlers.walk_children(element.node).content;
            let trimmed = content.trim();
            return Some(htmd::element_handler::HandlerResult::from(format!(
                "\n[^{base}]: {trimmed}\n"
            )));
        }

        // Footnote container: div#footnotes -> walk children
        if footnotes::is_canonical_footnotes_div(&attrs) {
            let content = handlers.walk_children(element.node).content;
            return Some(htmd::element_handler::HandlerResult::from(content));
        }
    }

    handlers.fallback(element)
}

/// Handle `<a>` elements: suppress footnote backref links.
fn handle_anchor_element(
    handlers: &dyn htmd::element_handler::Handlers,
    element: htmd::Element,
) -> Option<htmd::element_handler::HandlerResult> {
    let attrs = collect_attrs(&element);
    if footnotes::is_footnote_backref(&attrs) {
        return Some(htmd::element_handler::HandlerResult::from(String::new()));
    }
    // Fall through to default anchor handling
    handlers.fallback(element)
}

/// Collect attributes from an htmd Element into (name, value) pairs.
fn collect_attrs(element: &htmd::Element) -> Vec<(String, String)> {
    element
        .attrs
        .iter()
        .map(|a| (a.name.local.as_ref().to_string(), a.value.to_string()))
        .collect()
}

/// Reverse markdown escaping inside LaTeX math delimiters.
///
/// htmd escapes markdown-special characters (`_`, `\`, `*`, etc.)
/// everywhere, including inside `$...$` and `$$...$$` math
/// delimiters. This breaks LaTeX: `$\sum_{i=0}^n$` becomes
/// `$\\sum\_{i=0}^n$`. We restore the original LaTeX by removing
/// the extra backslash escapes within math regions.
fn unescape_latex_delimiters(md: &str) -> String {
    use regex::Regex;
    use std::sync::LazyLock;

    // Match $$...$$ (block) and $...$ (inline), non-greedy.
    // The block pattern must come first so `$$` isn't parsed as
    // two inline delimiters.
    static MATH_RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"\$\$[\s\S]+?\$\$|\$[^\$\n]+?\$").expect("latex delimiter regex is valid")
    });

    MATH_RE
        .replace_all(md, |caps: &regex::Captures| {
            let matched = &caps[0];
            unescape_latex(matched)
        })
        .into_owned()
}

/// Remove backslash escapes that htmd inserts for markdown-special
/// characters. Only the escapes htmd adds are reversed: `\\` -> `\`,
/// `\_` -> `_`, `\*` -> `*`, `\[` -> `[`, `\]` -> `]`, `\{` -> `{`,
/// `\}` -> `}`, `\#` -> `#`, `\~` -> `~`, `\|` -> `|`, `\>` -> `>`.
fn unescape_latex(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\\'
            && let Some(&next) = chars.peek()
            && matches!(
                next,
                '\\' | '_'
                    | '*'
                    | '['
                    | ']'
                    | '{'
                    | '}'
                    | '#'
                    | '~'
                    | '|'
                    | '>'
                    | '.'
                    | '!'
                    | '-'
                    | '('
                    | ')'
                    | '+'
            )
        {
            result.push(next);
            chars.next();
            continue;
        }
        result.push(ch);
    }
    result
}

/// Prevent `!` at the end of a word from merging with markdown image
/// syntax `![`. For example, `Yey!![IMG](url)` becomes
/// `Yey! ![IMG](url)`.
fn fix_bang_image_collision(md: &str) -> String {
    use regex::Regex;
    use std::sync::LazyLock;
    // Insert a space when text ending with `!` (preceded by a word
    // char) runs into markdown image `![` or linked image `[![`.
    // `Yey!![IMG](url)` -> `Yey! ![IMG](url)`
    // `Hello![![photo](url)](url)` -> `Hello! [![photo](url)](url)`
    static BANG_IMAGE_RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(\w!)\[?!\[").expect("fix_bang_image_collision regex is valid")
    });
    BANG_IMAGE_RE
        .replace_all(md, |caps: &regex::Captures| {
            let prefix = &caps[1];
            let matched = &caps[0];
            let rest = &matched[prefix.len()..];
            format!("{prefix} {rest}")
        })
        .into_owned()
}
