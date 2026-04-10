use std::sync::LazyLock;

use ego_tree::NodeId;
use regex::Regex;
use scraper::Html;

use crate::dom;

static CONTENT_CLASS_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(content|article|post)")
        .unwrap_or_else(|_| Regex::new("a]^").expect("infallible fallback"))
});

static LIKELY_CONTENT_CLASS_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(article|content|entry|main|post|story)")
        .unwrap_or_else(|_| Regex::new("a]^").expect("infallible fallback"))
});

static NAV_INDICATORS_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(concat!(
        r"(?i)",
        r"(advertisement|all rights reserved|banner|cookie|",
        r"comments|copyright|follow me|follow us|footer|",
        r"header|homepage|login|menu|more articles|",
        r"more like this|most read|nav|navigation|",
        r"newsletter|popular|privacy|recommended|register|",
        r"related|responses|share|sidebar|sign in|sign up|",
        r"signup|social|sponsored|subscribe|terms|trending)"
    ))
    .unwrap_or_else(|_| Regex::new("a]^").expect("infallible fallback"))
});

static NON_CONTENT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(concat!(
        r"(?i)",
        r"(advert|ad-|ads|banner|cookie|copyright|footer|",
        r"header|menu|nav|newsletter|popup|promo|related|",
        r"share|sidebar|social|sponsor|subscribe|widget)"
    ))
    .unwrap_or_else(|_| Regex::new("a]^").expect("infallible fallback"))
});

static BYLINE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\bBy\s+[A-Z][a-z]+")
        .unwrap_or_else(|_| Regex::new("a]^").expect("infallible fallback"))
});

static DATE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(concat!(
        r"(?i)",
        r"(\d{1,2}[/\-]\d{1,2}[/\-]\d{2,4}|",
        r"\b(?:jan|feb|mar|apr|may|jun|jul|aug|sep|oct|nov|dec)",
        r"[a-z]*\s+\d{1,2})"
    ))
    .unwrap_or_else(|_| Regex::new("a]^").expect("infallible fallback"))
});

static AUTHOR_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b(?:by|written by|author:)\s+")
        .unwrap_or_else(|_| Regex::new("a]^").expect("infallible fallback"))
});

static SENTENCE_PUNCT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"[.!?]").unwrap_or_else(|_| Regex::new("a]^").expect("infallible fallback"))
});

/// CSS selector for elements that indicate real content and should be
/// protected from removal.
pub const CONTENT_ELEMENT_SELECTOR: &str = concat!(
    "math, .katex, .MathJax, mjx-container, ",
    "code, pre, table, img, blockquote, figure"
);

const SOCIAL_DOMAINS: &[&str] = &["twitter.com", "github.com", "linkedin.com", "facebook.com"];

const FOOTNOTE_REF_SELECTOR: &str = "a[href^=\"#fn\"], sup.reference";

const FOOTNOTE_LIST_SELECTOR: &str = "ol.footnotes, div.footnotes";

const MATH_SELECTOR: &str = ".katex, .MathJax, math, mjx-container";

fn class_and_id(html: &Html, node_id: NodeId) -> String {
    let class = dom::get_attr(html, node_id, "class").unwrap_or_default();
    let id = dom::get_attr(html, node_id, "id").unwrap_or_default();
    format!("{class} {id}")
}

fn link_text_ratio(html: &Html, node_id: NodeId) -> f64 {
    let total_text = dom::text_content(html, node_id);
    let total_len = total_text.trim().len();
    if total_len == 0 {
        return 0.0;
    }
    let mut link_len = 0usize;
    for a_id in dom::descendant_elements_by_tag(html, node_id, "a") {
        link_len += dom::text_content(html, a_id).trim().len();
    }
    #[allow(clippy::cast_precision_loss)]
    let ratio = link_len as f64 / total_len as f64;
    ratio
}

fn count_commas(text: &str) -> usize {
    text.chars().filter(|&c| c == ',').count()
}

/// Check if element has right-aligned styling.
fn is_right_aligned(html: &Html, node_id: NodeId) -> bool {
    if let Some(align) = dom::get_attr(html, node_id, "align")
        && align.eq_ignore_ascii_case("right")
    {
        return true;
    }
    if let Some(style) = dom::get_attr(html, node_id, "style") {
        let lower = style.to_ascii_lowercase();
        return lower.contains("float:right")
            || lower.contains("float: right")
            || lower.contains("text-align:right")
            || lower.contains("text-align: right");
    }
    false
}

/// Detect center table cell layout (table cell with large width,
/// content/article class).
fn is_center_table_cell(html: &Html, node_id: NodeId) -> bool {
    let tds = dom::descendant_elements_by_tag(html, node_id, "td");
    for td_id in tds {
        let class_id = class_and_id(html, td_id);
        if !CONTENT_CLASS_RE.is_match(&class_id) {
            continue;
        }
        if let Some(width) = dom::get_attr(html, td_id, "width")
            && let Ok(w) = width.trim_end_matches("px").parse::<u32>()
            && w > 400
        {
            return true;
        }
    }
    false
}

/// Score an element for content likelihood.
/// Higher = more likely to be main content.
#[allow(clippy::cast_precision_loss)]
pub fn score_element(html: &Html, node_id: NodeId) -> f64 {
    let text = dom::text_content(html, node_id);
    let words = dom::count_words(&text);
    let paragraphs = dom::descendant_elements_by_tag(html, node_id, "p").len();
    let commas = count_commas(&text);
    let images = dom::descendant_elements_by_tag(html, node_id, "img").len();

    let mut score = words as f64;
    score += paragraphs as f64 * 10.0;
    score += commas as f64;

    let image_density = images as f64 / words.max(1) as f64;
    score -= image_density * 3.0;

    let class_id = class_and_id(html, node_id);
    if CONTENT_CLASS_RE.is_match(&class_id) {
        score += 15.0;
    }

    let ratio = link_text_ratio(html, node_id);
    score *= 1.0 - ratio.min(0.5);

    score += score_alignment_and_metadata(html, node_id, &text);
    score += score_footnotes_and_tables(html, node_id);

    score
}

/// Bonus for right-alignment, date patterns, and author patterns.
fn score_alignment_and_metadata(html: &Html, node_id: NodeId, text: &str) -> f64 {
    let mut bonus = 0.0;
    if is_right_aligned(html, node_id) {
        bonus += 5.0;
    }
    if DATE_RE.is_match(text) {
        bonus += 10.0;
    }
    if AUTHOR_RE.is_match(text) {
        bonus += 10.0;
    }
    bonus
}

/// Bonus for footnotes, center table cells, and penalty for nested
/// tables.
#[allow(clippy::cast_precision_loss)]
fn score_footnotes_and_tables(html: &Html, node_id: NodeId) -> f64 {
    let mut bonus = 0.0;
    if dom::has_descendant_matching(html, node_id, FOOTNOTE_REF_SELECTOR) {
        bonus += 10.0;
    }
    if dom::has_descendant_matching(html, node_id, FOOTNOTE_LIST_SELECTOR) {
        bonus += 10.0;
    }
    let nested_tables = dom::descendant_elements_by_tag(html, node_id, "table").len();
    bonus -= nested_tables as f64 * 5.0;
    if is_center_table_cell(html, node_id) {
        bonus += 10.0;
    }
    bonus
}

/// Find the best-scoring element from a list, above
/// `min_score` threshold.
#[must_use]
pub fn find_best_element(html: &Html, elements: &[NodeId], min_score: f64) -> Option<NodeId> {
    let mut best: Option<(NodeId, f64)> = None;
    for &id in elements {
        let score = score_element(html, id);
        if score < min_score {
            continue;
        }
        let dominated = best.is_some_and(|(_, best_score)| score <= best_score);
        if !dominated {
            best = Some((id, score));
        }
    }
    best.map(|(id, _)| id)
}

/// Check if an element is likely content (should be preserved).
pub fn is_likely_content(html: &Html, node_id: NodeId) -> bool {
    if let Some(role) = dom::get_attr(html, node_id, "role")
        && (role == "article" || role == "main")
    {
        return true;
    }

    let class_id = class_and_id(html, node_id);
    if LIKELY_CONTENT_CLASS_RE.is_match(&class_id) {
        return true;
    }

    if has_structural_content(html, node_id) {
        return true;
    }

    let text = dom::text_content(html, node_id);
    let word_count = dom::count_words(&text);

    if should_reject_as_non_content(html, node_id, word_count) {
        return false;
    }

    if word_count > 100 {
        return true;
    }

    let paragraph_count = dom::descendant_elements_by_tag(html, node_id, "p").len();
    let list_item_count = dom::descendant_elements_by_tag(html, node_id, "li").len();
    if word_count > 50 && (paragraph_count + list_item_count) > 1 {
        return true;
    }
    if word_count >= 10
        && SENTENCE_PUNCT_RE.is_match(&text)
        && dom::link_density(html, node_id) < 0.1
    {
        return true;
    }

    false
}

/// Checks that should cause early rejection from `is_likely_content`.
fn should_reject_as_non_content(html: &Html, node_id: NodeId, word_count: usize) -> bool {
    let has_nav = has_nav_heading(html, node_id);

    if has_nav && word_count < 200 {
        return true;
    }
    if has_nav && dom::link_density(html, node_id) > 0.2 {
        return true;
    }
    if is_card_grid(html, node_id) {
        return true;
    }
    if word_count < 80 && has_social_profile_links(html, node_id) {
        return true;
    }
    false
}

/// Check if element contains a heading with navigation-like text.
fn has_nav_heading(html: &Html, node_id: NodeId) -> bool {
    let heading_tags = ["h1", "h2", "h3", "h4", "h5", "h6"];
    for tag in &heading_tags {
        for h_id in dom::descendant_elements_by_tag(html, node_id, tag) {
            let h_text = dom::text_content(html, h_id);
            if NAV_INDICATORS_RE.is_match(&h_text) {
                return true;
            }
        }
    }
    false
}

/// Detect card grid pattern: 3+ headings, 2+ images, sparse prose.
#[allow(clippy::cast_precision_loss)]
fn is_card_grid(html: &Html, node_id: NodeId) -> bool {
    let mut heading_count = 0usize;
    for tag in &["h1", "h2", "h3", "h4", "h5", "h6"] {
        heading_count += dom::descendant_elements_by_tag(html, node_id, tag).len();
    }
    if heading_count < 3 {
        return false;
    }
    let image_count = dom::descendant_elements_by_tag(html, node_id, "img").len();
    if image_count < 2 {
        return false;
    }
    let text = dom::text_content(html, node_id);
    let total_words = dom::count_words(&text);
    let words_per_heading = total_words as f64 / heading_count as f64;
    words_per_heading < 20.0
}

/// Check if element has links to social profile domains.
fn has_social_profile_links(html: &Html, node_id: NodeId) -> bool {
    let hrefs = dom::collect_link_hrefs(html, node_id);
    hrefs
        .iter()
        .any(|href| SOCIAL_DOMAINS.iter().any(|domain| href.contains(domain)))
}

fn has_structural_content(html: &Html, node_id: NodeId) -> bool {
    let tags = ["pre", "table", "figure", "picture"];
    for tag in &tags {
        if !dom::descendant_elements_by_tag(html, node_id, tag).is_empty() {
            return true;
        }
    }
    // Also check for math, code, and blockquote elements
    if dom::has_descendant_matching(html, node_id, MATH_SELECTOR) {
        return true;
    }
    if !dom::descendant_elements_by_tag(html, node_id, "code").is_empty() {
        return true;
    }
    if !dom::descendant_elements_by_tag(html, node_id, "blockquote").is_empty() {
        return true;
    }
    false
}

/// Score a non-content block. Negative score = should be removed.
#[allow(clippy::cast_precision_loss)]
pub fn score_non_content(html: &Html, node_id: NodeId) -> f64 {
    // Skip scoring for footnote lists — preserve them
    if dom::has_descendant_matching(html, node_id, FOOTNOTE_LIST_SELECTOR) {
        return 0.0;
    }

    let text = dom::text_content(html, node_id);
    let word_count = dom::count_words(&text);

    if word_count < 3 {
        return 0.0;
    }

    let mut score = count_commas(&text) as f64;

    let nav_matches = NAV_INDICATORS_RE.find_iter(&text).count();
    score -= nav_matches as f64 * 10.0;

    if dom::link_density(html, node_id) > 0.5 {
        score -= 15.0;
    }

    score = apply_link_heavy_penalty(html, node_id, &text, score);

    let class_id = class_and_id(html, node_id);
    let pattern_matches = NON_CONTENT_RE.find_iter(&class_id).count();
    score -= pattern_matches as f64 * 8.0;

    if word_count < 15 && BYLINE_RE.is_match(&text) && DATE_RE.is_match(&text) {
        score -= 10.0;
    }

    score += score_social_and_card_penalties(html, node_id, word_count);

    score
}

/// Additional non-content penalties for social links and card grids.
fn score_social_and_card_penalties(html: &Html, node_id: NodeId, word_count: usize) -> f64 {
    let mut penalty = 0.0;
    if word_count < 80 && has_social_profile_links(html, node_id) {
        penalty -= 15.0;
    }
    if is_card_grid(html, node_id) {
        penalty -= 15.0;
    }
    penalty
}

fn apply_link_heavy_penalty(html: &Html, node_id: NodeId, text: &str, mut score: f64) -> f64 {
    let links = dom::descendant_elements_by_tag(html, node_id, "a");
    let words = dom::count_words(text);
    if links.len() > 1 && words < 80 && link_text_ratio(html, node_id) > 0.8 {
        score -= 15.0;
    }
    score
}
