use std::sync::LazyLock;

use ego_tree::NodeId;
use regex::Regex;
use scraper::{Html, Node};

use crate::dom;

static CONTENT_CLASS_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(content|article|post)")
        .unwrap_or_else(|_| Regex::new("a]^").expect("infallible fallback"))
});

// Fix #4: added admonition, image, img, font, figure, figcaption, pre, table
static LIKELY_CONTENT_CLASS_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(concat!(
        r"(?i)(admonition|article|content|entry|figcaption|",
        r"figure|font|image|img|main|post|pre|story|table)"
    ))
    .unwrap_or_else(|_| Regex::new("a]^").expect("infallible fallback"))
});

// Fix #7: each nav indicator wrapped with \b word boundaries
static NAV_INDICATORS_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(concat!(
        r"(?i)",
        r"(\b(?:advertisement)\b|\b(?:all rights reserved)\b|",
        r"\b(?:banner)\b|\b(?:cookie)\b|",
        r"\b(?:comments)\b|\b(?:copyright)\b|",
        r"\b(?:follow me)\b|\b(?:follow us)\b|\b(?:footer)\b|",
        r"\b(?:header)\b|\b(?:homepage)\b|\b(?:login)\b|",
        r"\b(?:menu)\b|\b(?:more articles)\b|",
        r"\b(?:more like this)\b|\b(?:most read)\b|",
        r"\b(?:nav)\b|\b(?:navigation)\b|",
        r"\b(?:newsletter)\b|\b(?:popular)\b|\b(?:privacy)\b|",
        r"\b(?:recommended)\b|\b(?:register)\b|",
        r"\b(?:related)\b|\b(?:responses)\b|\b(?:share)\b|",
        r"\b(?:sidebar)\b|\b(?:sign in)\b|\b(?:sign up)\b|",
        r"\b(?:signup)\b|\b(?:social)\b|\b(?:sponsored)\b|",
        r"\b(?:subscribe)\b|\b(?:terms)\b|\b(?:trending)\b)"
    ))
    .unwrap_or_else(|_| Regex::new("a]^").expect("infallible fallback"))
});

// Fix #8: add homepage, popular, privacy, recommended, rights, terms,
// trending; remove popup, promo; change sponsor to sponsored
static NON_CONTENT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(concat!(
        r"(?i)",
        r"(advert|ad-|ads|banner|cookie|copyright|footer|",
        r"header|homepage|menu|nav|newsletter|popular|privacy|",
        r"recommended|related|rights|share|sidebar|social|",
        r"sponsored|subscribe|terms|trending|widget)"
    ))
    .unwrap_or_else(|_| Regex::new("a]^").expect("infallible fallback"))
});

// Fix #3: case-sensitive, match capital letter after "By "
static BYLINE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\bBy\s+[A-Z]").unwrap_or_else(|_| Regex::new("a]^").expect("infallible fallback"))
});

// Loose date regex for byline detection (no year required)
static DATE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(concat!(
        r"(?i)",
        r"(\d{1,2}[/\-]\d{1,2}[/\-]\d{2,4}|",
        r"\b(?:jan|feb|mar|apr|may|jun|jul|aug|sep|oct|nov|dec)",
        r"[a-z]*\s+\d{1,2})"
    ))
    .unwrap_or_else(|_| Regex::new("a]^").expect("infallible fallback"))
});

// Fix #1: strict date regex for scoring (requires 4-digit year)
static CONTENT_DATE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(concat!(
        r"(?i)\b(?:",
        r"(?:Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec)",
        r"[a-z]*\s+\d{1,2},?\s+\d{4}",
        r"|",
        r"\d{1,2}(?:st|nd|rd|th)?\s+",
        r"(?:Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec)",
        r"[a-z]*,?\s+\d{4}",
        r")\b"
    ))
    .unwrap_or_else(|_| Regex::new("a]^").expect("infallible fallback"))
});

// Fix #2: author regex requires trailing name characters
static AUTHOR_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b(?:by|written by|author:)\s+[A-Za-z\s]+\b")
        .unwrap_or_else(|_| Regex::new("a]^").expect("infallible fallback"))
});

static SENTENCE_PUNCT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"[.!?]").unwrap_or_else(|_| Regex::new("a]^").expect("infallible fallback"))
});

// Fix #9: social profile regex with negative lookahead (fancy_regex)
static SOCIAL_PROFILE_RE: LazyLock<fancy_regex::Regex> = LazyLock::new(|| {
    fancy_regex::Regex::new(concat!(
        r"(?i)\b(",
        r"linkedin\.com/(in|company)/|",
        r"twitter\.com/(?!intent\b)\w|",
        r"x\.com/(?!intent\b)\w|",
        r"facebook\.com/(?!share\b)\w|",
        r"instagram\.com/\w|",
        r"threads\.net/\w|",
        r"mastodon\.\w",
        r")"
    ))
    .unwrap_or_else(|_| fancy_regex::Regex::new("a]^").expect("infallible fallback"))
});

/// CSS selector for elements that indicate real content and should be
/// protected from removal.
pub const CONTENT_ELEMENT_SELECTOR: &str = concat!(
    "math, .katex, .MathJax, mjx-container, ",
    "code, pre, table, img, blockquote, figure"
);

const FOOTNOTE_REF_SELECTOR: &str = "a[href^=\"#fn\"], sup.reference";

const FOOTNOTE_LIST_SELECTOR: &str = "ol.footnotes, div.footnotes";

fn class_and_id(html: &Html, node_id: NodeId) -> String {
    let class = dom::get_attr(html, node_id, "class").unwrap_or_default();
    let id = dom::get_attr(html, node_id, "id").unwrap_or_default();
    format!("{class} {id}")
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

/// Fix #11: check if the scored element itself is a `<td>` inside a
/// layout table (width>400), with align=center or content class,
/// and is not the first or last child of its parent.
fn is_layout_table_cell(html: &Html, node_id: NodeId) -> bool {
    let Some(tag) = dom::tag_name(html, node_id) else {
        return false;
    };
    if tag != "td" {
        return false;
    }
    // Check align=center or content class
    let has_center =
        dom::get_attr(html, node_id, "align").is_some_and(|a| a.eq_ignore_ascii_case("center"));
    let class_id = class_and_id(html, node_id);
    if !has_center && !CONTENT_CLASS_RE.is_match(&class_id) {
        return false;
    }
    // Must not be first or last child
    if let Some(parent_id) = dom::parent_element(html, node_id) {
        let siblings = dom::child_elements(html, parent_id);
        if siblings.first() == Some(&node_id) || siblings.last() == Some(&node_id) {
            return false;
        }
        // Check containing table has width > 400
        if let Some(table_tag) = dom::tag_name(html, parent_id)
            && table_tag == "table"
        {
            return table_has_large_width(html, parent_id);
        }
        // Walk up to find the table
        let mut cur = Some(parent_id);
        while let Some(id) = cur {
            if dom::tag_name(html, id).as_deref() == Some("table") {
                return table_has_large_width(html, id);
            }
            cur = dom::parent_element(html, id);
        }
    }
    false
}

fn table_has_large_width(html: &Html, table_id: NodeId) -> bool {
    dom::get_attr(html, table_id, "width")
        .and_then(|w| w.trim_end_matches("px").parse::<u32>().ok())
        .is_some_and(|w| w > 400)
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

    // Use pre-computed text to avoid redundant tree walk
    let ratio = dom::link_density_with_text(html, node_id, &text);
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
    // Fix #1: use strict date regex requiring 4-digit year
    if CONTENT_DATE_RE.is_match(text) {
        bonus += 10.0;
    }
    if AUTHOR_RE.is_match(text) {
        bonus += 10.0;
    }
    bonus
}

/// Bonus for footnotes, layout table cells, and penalty for nested
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
    // Fix #11: check if element itself is a layout table cell
    if is_layout_table_cell(html, node_id) {
        bonus += 10.0;
    }
    bonus
}

/// Find the best-scoring element from a list, above
/// `min_score` threshold.
/// Fix #14: on tie, pick the last element (>= instead of >).
#[must_use]
pub fn find_best_element(html: &Html, elements: &[NodeId], min_score: f64) -> Option<NodeId> {
    let mut best: Option<(NodeId, f64)> = None;
    for &id in elements {
        let score = score_element(html, id);
        if score < min_score {
            continue;
        }
        let dominated = best.is_some_and(|(_, best_score)| score < best_score);
        if !dominated {
            best = Some((id, score));
        }
    }
    best.map(|(id, _)| id)
}

/// Check if an element is likely content (should be preserved).
pub fn is_likely_content(html: &Html, node_id: NodeId) -> bool {
    // Cheap attribute checks first
    if let Some(role) = dom::get_attr(html, node_id, "role")
        && (role == "article" || role == "main" || role == "contentinfo")
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

    // Compute text once and reuse
    let text = dom::text_content(html, node_id);
    let word_count = dom::count_words(&text);

    // Fix #5: nav heading rejection with 200-999 word tier
    if word_count < 1000 && has_nav_heading(html, node_id) {
        if word_count < 200 {
            return false;
        }
        let link_density = dom::link_density_with_text(html, node_id, &text);
        if link_density > 0.2 {
            return false;
        }
    }
    if is_card_grid(html, node_id) {
        return false;
    }
    if word_count < 80 && has_social_profile_links(html, node_id) {
        return false;
    }

    if word_count > 100 {
        return true;
    }

    let paragraph_count = dom::descendant_elements_by_tag(html, node_id, "p").len();
    let list_item_count = dom::descendant_elements_by_tag(html, node_id, "li").len();

    // Fix #6: words > 30 with at least one content block
    if word_count > 30 && (paragraph_count + list_item_count) > 0 {
        return true;
    }
    if word_count > 50 && (paragraph_count + list_item_count) > 1 {
        return true;
    }
    if word_count >= 10
        && SENTENCE_PUNCT_RE.is_match(&text)
        && dom::link_density_with_text(html, node_id, &text) < 0.1
    {
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

/// Fix #10: detect card grid pattern.
/// Only check h2-h4, skip if words < 3 or >= 500, subtract heading
/// word count from total before computing prose-per-heading.
#[allow(clippy::cast_precision_loss)]
fn is_card_grid(html: &Html, node_id: NodeId) -> bool {
    let text = dom::text_content(html, node_id);
    let total_words = dom::count_words(&text);
    if !(3..500).contains(&total_words) {
        return false;
    }
    let mut heading_ids = Vec::new();
    for tag in &["h2", "h3", "h4"] {
        heading_ids.extend(dom::descendant_elements_by_tag(html, node_id, tag));
    }
    if heading_ids.len() < 3 {
        return false;
    }
    let image_count = dom::descendant_elements_by_tag(html, node_id, "img").len();
    if image_count < 2 {
        return false;
    }
    let heading_words: usize = heading_ids
        .iter()
        .map(|&h_id| dom::count_words(&dom::text_content(html, h_id)))
        .sum();
    let prose_words = total_words.saturating_sub(heading_words);
    let words_per_heading = prose_words as f64 / heading_ids.len() as f64;
    words_per_heading < 20.0
}

/// Fix #9: check social profile links with full regex including
/// negative lookahead for intent/share URLs.
fn has_social_profile_links(html: &Html, node_id: NodeId) -> bool {
    let hrefs = dom::collect_link_hrefs(html, node_id);
    hrefs
        .iter()
        .any(|href| SOCIAL_PROFILE_RE.is_match(href).unwrap_or(false))
}

/// Check if any descendant has a structural content tag.
/// Uses a single tree walk instead of separate per-tag searches.
fn has_structural_content(html: &Html, node_id: NodeId) -> bool {
    has_descendant_with_tag(
        html,
        node_id,
        &[
            "pre",
            "table",
            "figure",
            "picture",
            "code",
            "blockquote",
            "math",
            "mjx-container",
        ],
    )
}

/// Walk descendants once, checking if any match the given tag names.
fn has_descendant_with_tag(html: &Html, node_id: NodeId, tags: &[&str]) -> bool {
    let Some(node_ref) = html.tree.get(node_id) else {
        return false;
    };
    for child in node_ref.children() {
        if let Node::Element(el) = child.value() {
            let tag = el.name.local.as_ref();
            if tags.contains(&tag) {
                return true;
            }
            // Also check for math classes
            if let Some(class) = el.attr("class")
                && (class.contains("katex") || class.contains("MathJax"))
            {
                return true;
            }
        }
        if has_descendant_with_tag(html, child.id(), tags) {
            return true;
        }
    }
    false
}

/// Score a non-content block. Negative score = should be removed.
#[allow(clippy::cast_precision_loss)]
pub fn score_non_content(html: &Html, node_id: NodeId) -> f64 {
    // Fix #12: skip if element itself, any ancestor, or any descendant
    // matches footnote list selectors
    if dom::self_or_ancestor_matches(html, node_id, FOOTNOTE_LIST_SELECTOR)
        || dom::has_descendant_matching(html, node_id, FOOTNOTE_LIST_SELECTOR)
    {
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

    // Reuse text for link density computation
    if dom::link_density_with_text(html, node_id, &text) > 0.5 {
        score -= 15.0;
    }

    score = apply_link_heavy_penalty(html, node_id, &text, score);

    let class_id = class_and_id(html, node_id);
    let pattern_matches = NON_CONTENT_RE.find_iter(&class_id).count();
    score -= pattern_matches as f64 * 8.0;

    if word_count < 15 && BYLINE_RE.is_match(&text) && DATE_RE.is_match(&text) {
        score -= 10.0;
    }

    let links = dom::descendant_elements_by_tag(html, node_id, "a").len();
    let lists = dom::descendant_elements_by_tag(html, node_id, "ul").len()
        + dom::descendant_elements_by_tag(html, node_id, "ol").len();
    if lists > 0 && links > lists * 3 {
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
    if links.len() > 1 && words < 80 && dom::link_density_with_text(html, node_id, text) > 0.8 {
        score -= 15.0;
    }
    score
}
