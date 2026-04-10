use std::time::Instant;

use ego_tree::NodeId;
use scraper::{Html, Node};

use crate::cleanup;
use crate::content;
use crate::dom;
use crate::metadata;
use crate::noscript;
use crate::patterns;
use crate::schema_org;
use crate::standardize;
use crate::types::{DebugInfo, DecruftOptions, DecruftResult, Removal};

/// Result of a single parse-internal pass.
struct ParseResult {
    content: String,
    word_count: usize,
    content_selector_path: String,
    removals: Vec<Removal>,
}

/// Parse HTML and extract clean, readable content.
#[must_use]
pub fn parse(html_str: &str, options: &DecruftOptions) -> DecruftResult {
    let start = Instant::now();

    let html = Html::parse_document(html_str);
    let schema_data = schema_org::extract_schema_org(&html);
    let meta_tags = cleanup::collect_meta_tags(&html);
    let meta = metadata::extract_metadata(&html, options.url.as_deref(), schema_data.as_ref());

    // ATTEMPT 1: Default settings
    let mut result = parse_internal(html_str, options);

    // RETRY 1: If word_count < 200, retry without partial selectors
    if result.word_count < 200 {
        result = retry_without_partials(html_str, options, &result);
    }

    // RETRY 2: If word_count < 50, retry without hidden element removal
    if result.word_count < 50 {
        result = retry_without_hidden(html_str, options, result);
    }

    // RETRY 3: If still < 50, retry without scoring/partials/patterns
    if result.word_count < 50 {
        result = retry_fully_relaxed(html_str, options, result);
    }

    // RETRY 4: Schema.org content fallback
    result = retry_schema_fallback(html_str, options, schema_data.as_ref(), result);

    let elapsed = start.elapsed();
    #[allow(clippy::cast_possible_truncation)]
    let parse_time_ms = elapsed.as_millis() as u64;

    build_result(
        result.content,
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
/// Only use the retry if it yields 2x+ improvement.
fn retry_without_partials(
    html_str: &str,
    options: &DecruftOptions,
    current: &ParseResult,
) -> ParseResult {
    let mut opts = options.clone();
    opts.remove_partial_selectors = false;
    let retry = parse_internal(html_str, &opts);
    if retry.word_count > current.word_count * 2 {
        retry
    } else {
        // Re-parse with original options to get owned result
        parse_internal(html_str, options)
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
    let retry = parse_internal(html_str, &opts);
    if retry.word_count > result.word_count {
        retry
    } else {
        result
    }
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
        result.content = schema_text;
        result.word_count = schema_wc;
    }
    result
}

/// Find the CSS selector for the largest hidden element with
/// substantial content (>= 30 words).
///
/// Checks elements with `[hidden]`, `[aria-hidden="true"]`,
/// `.hidden`, and `.invisible` attributes/classes.
fn find_largest_hidden_content_selector(html_str: &str) -> Option<String> {
    let html = Html::parse_document(html_str);
    let hidden_selectors = [
        "[hidden]",
        "[aria-hidden=\"true\"]",
        ".hidden",
        ".invisible",
    ];

    let mut best_selector: Option<String> = None;
    let mut best_word_count: usize = 0;

    for sel_str in &hidden_selectors {
        for id in dom::select_ids(&html, sel_str) {
            let text = dom::text_content(&html, id);
            let wc = dom::count_words(&text);
            if wc >= 30 && wc > best_word_count {
                best_word_count = wc;
                best_selector = Some(build_unique_selector(&html, id));
            }
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
    let plain = strip_tags_simple(html_text);
    let trimmed = plain.trim();

    let paragraph = trimmed.split("\n\n").next().unwrap_or(trimmed).trim();

    if paragraph.len() <= max_chars {
        paragraph.to_string()
    } else {
        paragraph[..max_chars].to_string()
    }
}

/// Simple tag stripping for schema text comparison.
fn strip_tags_simple(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    for ch in html.chars() {
        if ch == '<' {
            in_tag = true;
        } else if ch == '>' {
            in_tag = false;
            result.push(' ');
        } else if !in_tag {
            result.push(ch);
        }
    }
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
    if options.remove_images {
        cleanup::remove_all_images(html, main_content);
    }
    if options.remove_small_images {
        cleanup::remove_small_images(html, main_content);
    }
    if options.remove_hidden_elements {
        cleanup::remove_hidden_elements(html, main_content, removals, options.debug);
    }
    if options.remove_exact_selectors {
        cleanup::remove_exact_selectors(html, main_content, removals, options.debug);
    }
    if options.remove_partial_selectors {
        cleanup::remove_partial_selectors(html, main_content, removals, options.debug);
    }
    if options.remove_low_scoring {
        cleanup::score_and_remove(html, main_content, removals, options.debug);
    }
    if options.remove_content_patterns {
        patterns::remove_content_patterns(html, main_content, removals, options.debug);
    }
    if options.standardize {
        standardize::standardize_content(html, main_content, options.debug);
    }
    if let Some(ref url) = options.url {
        standardize::resolve_urls(html, main_content, url);
    }
    cleanup::deduplicate_images(html, main_content);
}

#[allow(clippy::too_many_arguments)]
fn build_result(
    content: String,
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
        title: meta.title,
        description: meta.description,
        domain: meta.domain,
        favicon: meta.favicon,
        image: meta.image,
        language: meta.language,
        parse_time_ms,
        published: meta.published,
        author: meta.author,
        site: meta.site_name,
        word_count,
        schema_org_data: schema_data,
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
