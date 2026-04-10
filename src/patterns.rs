use std::sync::LazyLock;

use ego_tree::NodeId;
use regex::Regex;
use scraper::Html;

use crate::dom;
use crate::types::Removal;

static READ_TIME_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\d+\s*min(ute)?s?\s*(read|to read)").expect("valid regex"));

static BYLINE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^(by|written by|author:)\s+\S").expect("valid regex"));

static DATE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)(jan|feb|mar|apr|may|jun|jul|aug|sep|oct|nov|dec)\w*\s+\d{1,2},?\s+\d{4}|\d{4}-\d{2}-\d{2}|\d{1,2}/\d{1,2}/\d{2,4}",
    )
    .expect("valid regex")
});

static BOILERPLATE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)(this article (appeared|was published|originally)|originally published|©\s*\d{4}|all rights reserved|comments|leave a reply|loading\.{3}|subscribe to|sign up for)",
    )
    .expect("valid regex")
});

static RELATED_HEADING_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)^(related\s*(posts|articles|stories)?|you might also like|recommended|more from|subscribe|follow us|share this|newsletter|stay updated|get the newsletter|don'?t miss out|join our community|keep reading)",
    )
    .expect("valid regex")
});

static AUTHOR_DATE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)^[A-Z][a-z]+(\s+[A-Z][a-z]+){0,3}\s*[|·•/,]\s*((jan|feb|mar|apr|may|jun|jul|aug|sep|oct|nov|dec)\w*\s+\d{1,2},?\s+\d{4}|\d{4}-\d{2}-\d{2}|\d{1,2}/\d{1,2}/\d{2,4})",
    )
    .expect("valid regex")
});

/// Matches text starting with a metadata label like "Date:", "Published:",
/// etc. These should not be treated as author bylines.
static METADATA_LABEL_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^(?:date|published|updated|posted|from|to|subject|sent|cc|bcc)\s*:")
        .expect("valid regex")
});

/// Weekday abbreviations that should not be mistaken for author names.
static WEEKDAY_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^(Mon|Tue|Wed|Thu|Fri|Sat|Sun)\b").expect("valid regex"));

static SOCIAL_COUNTER_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^\s*\d+\s*(likes?|comments?|shares?|retweets?|replies|reactions?)\s*$")
        .expect("valid regex")
});

static RELATED_INTRO_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^for more (on|about)\b").expect("valid regex"));

static TIMEZONE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)current time in\b").expect("valid regex"));

static PINNED_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^\s*pinned\s*$").expect("valid regex"));

/// Check if text starting with "author" looks like an actual author
/// list or label (e.g. "Authors: Name1, Name2" or "Authors:")
/// rather than a navigation widget (e.g. "Author bio").
fn looks_like_author_list(lower: &str) -> bool {
    // Standalone label "Authors:" or "Author:" - likely a descriptor
    // for adjacent content, not a widget
    if lower.trim_end() == "authors:" || lower.trim_end() == "author:" {
        return true;
    }
    // "Authors: Name1, Name2" or "Author: Name"
    if lower.starts_with("authors:") || lower.starts_with("author:") {
        return true;
    }
    // "Authors Name1, Name2" - plural with comma-separated names
    if lower.starts_with("authors") && lower.contains(',') {
        return true;
    }
    false
}

/// Remove content patterns: bylines, read time, boilerplate, related posts.
pub fn remove_content_patterns(
    html: &mut Html,
    main_content: NodeId,
    removals: &mut Vec<Removal>,
    debug: bool,
) {
    remove_read_time(html, main_content, removals, debug);
    remove_bylines(html, main_content, removals, debug);
    remove_boilerplate(html, main_content, removals, debug);
    remove_related_headings(html, main_content, removals, debug);
    remove_boundary_time_elements(html, main_content, removals, debug);
    remove_trailing_link_lists(html, main_content, removals, debug);
    remove_hero_header(html, main_content, removals, debug);
    remove_breadcrumb_lists(html, main_content, removals, debug);
    remove_promotional_banners(html, main_content, removals, debug);
    remove_article_metadata_header(html, main_content, removals, debug);
    remove_author_date_bylines(html, main_content, removals, debug);
    remove_standalone_dates(html, main_content, removals, debug);
    remove_blog_metadata_lists(html, main_content, removals, debug);
    remove_section_breadcrumbs(html, main_content, removals, debug);
    remove_trailing_external_links(html, main_content, removals, debug);
    remove_trailing_related_posts(html, main_content, removals, debug);
    remove_trailing_thin_sections(html, main_content, removals, debug);
    remove_newsletter_signups(html, main_content, removals, debug);
    remove_author_contact_blocks(html, main_content, removals, debug);
    remove_author_share_widgets(html, main_content, removals, debug);
    remove_social_counters(html, main_content, removals, debug);
    remove_related_intro_paragraphs(html, main_content, removals, debug);
    remove_related_post_card_grids(html, main_content, removals, debug);
    remove_pinned_labels(html, main_content, removals, debug);
    remove_timezone_widgets(html, main_content, removals, debug);
}

fn remove_read_time(
    html: &mut Html,
    main_content: NodeId,
    removals: &mut Vec<Removal>,
    debug: bool,
) {
    let tags = ["p", "span", "div", "time"];
    let mut to_remove = Vec::new();

    for tag in &tags {
        let elements = dom::descendant_elements_by_tag(html, main_content, tag);
        for node_id in elements {
            let text = dom::text_content(html, node_id);
            let word_count = dom::count_words(&text);
            if word_count <= 10 && READ_TIME_RE.is_match(&text) {
                record_removal(removals, debug, "read time", &text);
                to_remove.push(node_id);
            }
        }
    }

    for id in to_remove {
        dom::remove_node(html, id);
    }
}

fn remove_bylines(html: &mut Html, main_content: NodeId, removals: &mut Vec<Removal>, debug: bool) {
    let tags = ["p", "span", "div"];
    let mut to_remove = Vec::new();

    for tag in &tags {
        let elements = dom::descendant_elements_by_tag(html, main_content, tag);
        for node_id in elements {
            let text = dom::text_content(html, node_id);
            if text.len() > 600 {
                continue;
            }
            if BYLINE_RE.is_match(text.trim()) {
                let word_count = dom::count_words(&text);
                if word_count <= 15 {
                    record_removal(removals, debug, "byline", &text);
                    to_remove.push(node_id);
                }
            }
        }
    }

    for id in to_remove {
        dom::remove_node(html, id);
    }
}

fn remove_boilerplate(
    html: &mut Html,
    main_content: NodeId,
    removals: &mut Vec<Removal>,
    debug: bool,
) {
    let tags = ["p", "div", "span"];
    let mut to_remove = Vec::new();

    for tag in &tags {
        let elements = dom::descendant_elements_by_tag(html, main_content, tag);
        for node_id in elements {
            let text = dom::text_content(html, node_id);
            let word_count = dom::count_words(&text);
            if word_count <= 30 && BOILERPLATE_RE.is_match(&text) {
                record_removal(removals, debug, "boilerplate", &text);
                to_remove.push(node_id);
            }
        }
    }

    for id in to_remove {
        dom::remove_node(html, id);
    }
}

fn remove_related_headings(
    html: &mut Html,
    main_content: NodeId,
    removals: &mut Vec<Removal>,
    debug: bool,
) {
    let heading_tags = ["h1", "h2", "h3", "h4", "h5", "h6"];
    let mut to_remove = Vec::new();

    for tag in &heading_tags {
        let elements = dom::descendant_elements_by_tag(html, main_content, tag);
        for node_id in elements {
            let text = dom::text_content(html, node_id);
            if RELATED_HEADING_RE.is_match(text.trim()) {
                record_removal(removals, debug, "related heading", &text);
                to_remove.push(node_id);

                // Also remove the next sibling if it's a list or div
                if let Some(next) = next_element_sibling(html, node_id) {
                    let next_tag = dom::tag_name(html, next);
                    if next_tag.as_deref() == Some("ul")
                        || next_tag.as_deref() == Some("ol")
                        || next_tag.as_deref() == Some("div")
                    {
                        to_remove.push(next);
                    }
                }
            }
        }
    }

    for id in to_remove {
        dom::remove_node(html, id);
    }
}

fn remove_boundary_time_elements(
    html: &mut Html,
    main_content: NodeId,
    removals: &mut Vec<Removal>,
    debug: bool,
) {
    let time_elements = dom::descendant_elements_by_tag(html, main_content, "time");
    let mut to_remove = Vec::new();

    for time_id in time_elements {
        let text = dom::text_content(html, time_id);
        let word_count = dom::count_words(&text);
        if word_count > 5 {
            continue;
        }

        // Check if it's near the start or end of content
        if let Some(parent_id) = dom::parent_element(html, time_id) {
            let parent_tag = dom::tag_name(html, parent_id);
            if parent_tag.as_deref() == Some("p") || parent_tag.as_deref() == Some("div") {
                let parent_text = dom::text_content(html, parent_id);
                let parent_words = dom::count_words(&parent_text);
                if parent_words <= 8 && DATE_RE.is_match(&text) {
                    record_removal(removals, debug, "boundary time", &text);
                    to_remove.push(parent_id);
                }
            }
        }
    }

    for id in to_remove {
        dom::remove_node(html, id);
    }
}

fn remove_trailing_link_lists(
    html: &mut Html,
    main_content: NodeId,
    removals: &mut Vec<Removal>,
    debug: bool,
) {
    let children = dom::child_elements(html, main_content);
    let len = children.len();
    if len < 2 {
        return;
    }

    let mut to_remove = Vec::new();

    // Check last few children for link-dense sections
    let start = len.saturating_sub(3);
    for &child_id in &children[start..] {
        let tag = dom::tag_name(html, child_id);
        let is_section = matches!(tag.as_deref(), Some("div" | "section" | "aside"));
        if !is_section {
            continue;
        }

        let density = dom::link_density(html, child_id);
        let word_count = dom::count_words(&dom::text_content(html, child_id));
        if density > 0.5 && word_count < 100 {
            record_removal(
                removals,
                debug,
                "trailing link list",
                &dom::text_content(html, child_id),
            );
            to_remove.push(child_id);
        }
    }

    for id in to_remove {
        dom::remove_node(html, id);
    }
}

/// Rule 1: Hero header removal - containers wrapping h1 + time + author
/// with < 30 prose words at the start of content.
fn remove_hero_header(
    html: &mut Html,
    main_content: NodeId,
    removals: &mut Vec<Removal>,
    debug: bool,
) {
    let children = dom::child_elements(html, main_content);
    if children.is_empty() {
        return;
    }

    let mut to_remove = Vec::new();

    // Check first few children for hero header pattern
    let check_count = children.len().min(3);
    for &child_id in &children[..check_count] {
        let tag = dom::tag_name(html, child_id);
        if !matches!(tag.as_deref(), Some("div" | "header" | "section")) {
            continue;
        }

        let has_h1 = !dom::descendant_elements_by_tag(html, child_id, "h1").is_empty();
        let has_time = !dom::descendant_elements_by_tag(html, child_id, "time").is_empty();

        if !has_h1 || !has_time {
            continue;
        }

        let text = dom::text_content(html, child_id);
        let word_count = dom::count_words(&text);
        if word_count < 30 {
            record_removal(removals, debug, "hero header", &text);
            to_remove.push(child_id);
        }
    }

    for id in to_remove {
        dom::remove_node(html, id);
    }
}

/// Rule 2: Breadcrumb list detection - first ul/ol with internal-only
/// links to parent URL paths.
fn remove_breadcrumb_lists(
    html: &mut Html,
    main_content: NodeId,
    removals: &mut Vec<Removal>,
    debug: bool,
) {
    let children = dom::child_elements(html, main_content);
    let mut to_remove = Vec::new();

    // Only check the first few children
    let check_count = children.len().min(3);
    for &child_id in &children[..check_count] {
        let tag = dom::tag_name(html, child_id);
        if !matches!(tag.as_deref(), Some("ul" | "ol" | "nav")) {
            continue;
        }

        let hrefs = dom::collect_link_hrefs(html, child_id);
        if hrefs.is_empty() {
            continue;
        }

        let all_internal = hrefs
            .iter()
            .all(|h| h.starts_with('/') || h.starts_with('#') || h.starts_with('.'));
        let text = dom::text_content(html, child_id);
        let word_count = dom::count_words(&text);

        if all_internal && word_count <= 20 {
            record_removal(removals, debug, "breadcrumb list", &text);
            to_remove.push(child_id);
        }
    }

    for id in to_remove {
        dom::remove_node(html, id);
    }
}

/// Rule 3: Promotional banner links - block `<a>` elements with `<div>`
/// children before first h1, short text.
fn remove_promotional_banners(
    html: &mut Html,
    main_content: NodeId,
    removals: &mut Vec<Removal>,
    debug: bool,
) {
    let children = dom::child_elements(html, main_content);
    let mut to_remove = Vec::new();

    for &child_id in &children {
        let tag = dom::tag_name(html, child_id);

        // Stop at first heading
        if matches!(tag.as_deref(), Some("h1" | "h2" | "h3")) {
            break;
        }

        if tag.as_deref() != Some("a") {
            continue;
        }

        let has_div = !dom::descendant_elements_by_tag(html, child_id, "div").is_empty();
        let text = dom::text_content(html, child_id);
        let word_count = dom::count_words(&text);

        if has_div && word_count <= 15 {
            record_removal(removals, debug, "promotional banner", &text);
            to_remove.push(child_id);
        }
    }

    for id in to_remove {
        dom::remove_node(html, id);
    }
}

/// Rule 4: Article metadata header blocks - div with date, <= 10 words,
/// near start of content.
fn remove_article_metadata_header(
    html: &mut Html,
    main_content: NodeId,
    removals: &mut Vec<Removal>,
    debug: bool,
) {
    let children = dom::child_elements(html, main_content);
    let mut to_remove = Vec::new();

    let check_count = children.len().min(5);
    for &child_id in &children[..check_count] {
        let tag = dom::tag_name(html, child_id);
        if tag.as_deref() != Some("div") {
            continue;
        }

        let text = dom::text_content(html, child_id);
        let word_count = dom::count_words(&text);
        if word_count <= 10 && DATE_RE.is_match(&text) {
            record_removal(removals, debug, "article metadata header", &text);
            to_remove.push(child_id);
        }
    }

    for id in to_remove {
        dom::remove_node(html, id);
    }
}

/// Rule 5: Author + date bylines - combined name + date patterns.
///
/// Skips lines starting with metadata labels (Date:, Published:, etc.)
/// and lines where the "author" portion is just a weekday abbreviation
/// (Mon, Tue, Wed...), which would otherwise false-positive on
/// email-style headers like "Date: Wed, 08 Apr 2026".
fn remove_author_date_bylines(
    html: &mut Html,
    main_content: NodeId,
    removals: &mut Vec<Removal>,
    debug: bool,
) {
    let tags = ["p", "span", "div"];
    let mut to_remove = Vec::new();

    for tag in &tags {
        let elements = dom::descendant_elements_by_tag(html, main_content, tag);
        for node_id in elements {
            let text = dom::text_content(html, node_id);
            let trimmed = text.trim();
            let word_count = dom::count_words(&text);
            if word_count > 15 {
                continue;
            }
            if METADATA_LABEL_RE.is_match(trimmed) {
                continue;
            }
            if WEEKDAY_RE.is_match(trimmed) {
                continue;
            }
            if AUTHOR_DATE_RE.is_match(trimmed) {
                record_removal(removals, debug, "author date byline", &text);
                to_remove.push(node_id);
            }
        }
    }

    for id in to_remove {
        dom::remove_node(html, id);
    }
}

/// Rule 6: Standalone date elements - near start, <= 5 words, matching
/// date patterns.
fn remove_standalone_dates(
    html: &mut Html,
    main_content: NodeId,
    removals: &mut Vec<Removal>,
    debug: bool,
) {
    let children = dom::child_elements(html, main_content);
    let mut to_remove = Vec::new();

    let check_count = children.len().min(5);
    for &child_id in &children[..check_count] {
        let tag = dom::tag_name(html, child_id);
        if !matches!(tag.as_deref(), Some("p" | "span" | "div" | "time")) {
            continue;
        }

        let text = dom::text_content(html, child_id);
        let word_count = dom::count_words(&text);
        if word_count <= 5 && DATE_RE.is_match(&text) {
            record_removal(removals, debug, "standalone date", &text);
            to_remove.push(child_id);
        }
    }

    for id in to_remove {
        dom::remove_node(html, id);
    }
}

/// Rule 7: Blog metadata lists - short ul/ol/dl at boundaries, all
/// items <= 8 words, no sentence punctuation.
fn remove_blog_metadata_lists(
    html: &mut Html,
    main_content: NodeId,
    removals: &mut Vec<Removal>,
    debug: bool,
) {
    let children = dom::child_elements(html, main_content);
    let len = children.len();
    if len == 0 {
        return;
    }

    let mut to_remove = Vec::new();

    // Check first and last few children
    let boundary_ids = collect_boundary_ids(&children, 3);

    for &child_id in &boundary_ids {
        let tag = dom::tag_name(html, child_id);
        if !matches!(tag.as_deref(), Some("ul" | "ol" | "dl")) {
            continue;
        }

        let items = dom::child_elements(html, child_id);
        if items.is_empty() || items.len() > 10 {
            continue;
        }

        let all_short_no_punct = items.iter().all(|&item_id| {
            let text = dom::text_content(html, item_id);
            let wc = dom::count_words(&text);
            wc <= 8 && !text.contains('.') && !text.contains('?')
        });

        if all_short_no_punct {
            let text = dom::text_content(html, child_id);
            record_removal(removals, debug, "blog metadata list", &text);
            to_remove.push(child_id);
        }
    }

    for id in to_remove {
        dom::remove_node(html, id);
    }
}

/// Rule 8: Section breadcrumbs - links to parent URL paths within
/// content.
fn remove_section_breadcrumbs(
    html: &mut Html,
    main_content: NodeId,
    removals: &mut Vec<Removal>,
    debug: bool,
) {
    let nav_elements = dom::descendant_elements_by_tag(html, main_content, "nav");
    let mut to_remove = Vec::new();

    for nav_id in nav_elements {
        let hrefs = dom::collect_link_hrefs(html, nav_id);
        if hrefs.is_empty() {
            continue;
        }

        let all_internal = hrefs
            .iter()
            .all(|h| h.starts_with('/') || h.starts_with('.'));
        let text = dom::text_content(html, nav_id);
        let word_count = dom::count_words(&text);

        if all_internal && word_count <= 20 {
            record_removal(removals, debug, "section breadcrumb", &text);
            to_remove.push(nav_id);
        }
    }

    for id in to_remove {
        dom::remove_node(html, id);
    }
}

/// Rule 9: Trailing external link lists - heading + list of off-site
/// links at content end.
fn remove_trailing_external_links(
    html: &mut Html,
    main_content: NodeId,
    removals: &mut Vec<Removal>,
    debug: bool,
) {
    let children = dom::child_elements(html, main_content);
    let len = children.len();
    if len < 2 {
        return;
    }

    let mut to_remove = Vec::new();
    let start = len.saturating_sub(4);

    for i in start..len {
        let child_id = children[i];
        let tag = dom::tag_name(html, child_id);
        if !matches!(tag.as_deref(), Some("ul" | "ol")) {
            continue;
        }

        let hrefs = dom::collect_link_hrefs(html, child_id);
        if hrefs.len() < 2 {
            continue;
        }

        let all_external = hrefs
            .iter()
            .all(|h| h.starts_with("http://") || h.starts_with("https://"));
        if !all_external {
            continue;
        }

        let text = dom::text_content(html, child_id);
        record_removal(removals, debug, "trailing external links", &text);
        to_remove.push(child_id);

        // Also remove preceding heading if present
        if i > 0 {
            let prev_id = children[i - 1];
            let prev_tag = dom::tag_name(html, prev_id);
            if is_heading_tag(prev_tag.as_deref()) {
                to_remove.push(prev_id);
            }
        }
    }

    for id in to_remove {
        dom::remove_node(html, id);
    }
}

/// Rule 10: Trailing related posts blocks - section/div at end with
/// link-dense paragraphs.
fn remove_trailing_related_posts(
    html: &mut Html,
    main_content: NodeId,
    removals: &mut Vec<Removal>,
    debug: bool,
) {
    let children = dom::child_elements(html, main_content);
    let len = children.len();
    if len < 2 {
        return;
    }

    let mut to_remove = Vec::new();
    let start = len.saturating_sub(3);

    for &child_id in &children[start..] {
        let tag = dom::tag_name(html, child_id);
        if !matches!(tag.as_deref(), Some("div" | "section" | "aside" | "footer")) {
            continue;
        }

        let paragraphs = dom::descendant_elements_by_tag(html, child_id, "p");
        if paragraphs.is_empty() {
            continue;
        }

        let link_dense_count = paragraphs
            .iter()
            .filter(|&&p_id| dom::link_density(html, p_id) > 0.5)
            .count();

        #[expect(clippy::cast_precision_loss)]
        let ratio = link_dense_count as f64 / paragraphs.len() as f64;
        if ratio > 0.5 {
            let text = dom::text_content(html, child_id);
            record_removal(removals, debug, "trailing related posts", &text);
            to_remove.push(child_id);
        }
    }

    for id in to_remove {
        dom::remove_node(html, id);
    }
}

/// Rule 11: Trailing thin sections - last children with heading but
/// < 25 words each, < 15% of total word count.
fn remove_trailing_thin_sections(
    html: &mut Html,
    main_content: NodeId,
    removals: &mut Vec<Removal>,
    debug: bool,
) {
    let children = dom::child_elements(html, main_content);
    let len = children.len();
    if len < 3 {
        return;
    }

    let total_text = dom::text_content(html, main_content);
    let total_words = dom::count_words(&total_text);
    if total_words == 0 {
        return;
    }

    let mut to_remove = Vec::new();
    let mut thin_words = 0usize;

    // Walk backwards from end
    for &child_id in children.iter().rev() {
        let text = dom::text_content(html, child_id);
        let word_count = dom::count_words(&text);

        if word_count >= 25 {
            break;
        }

        let has_heading = ["h1", "h2", "h3", "h4", "h5", "h6"]
            .iter()
            .any(|t| !dom::descendant_elements_by_tag(html, child_id, t).is_empty());

        if !has_heading {
            break;
        }

        thin_words += word_count;

        #[expect(clippy::cast_precision_loss)]
        let pct = thin_words as f64 / total_words as f64;
        if pct >= 0.15 {
            break;
        }

        record_removal(removals, debug, "trailing thin section", &text);
        to_remove.push(child_id);
    }

    for id in to_remove {
        dom::remove_node(html, id);
    }
}

/// Rule 12: Newsletter signups - containers with "subscribe" +
/// "newsletter" text patterns.
fn remove_newsletter_signups(
    html: &mut Html,
    main_content: NodeId,
    removals: &mut Vec<Removal>,
    debug: bool,
) {
    let tags = ["div", "section", "aside", "form"];
    let mut to_remove = Vec::new();

    for tag in &tags {
        let elements = dom::descendant_elements_by_tag(html, main_content, tag);
        for node_id in elements {
            let text = dom::text_content(html, node_id);
            let lower = text.to_lowercase();
            let word_count = dom::count_words(&text);

            if word_count > 80 {
                continue;
            }

            let has_subscribe = lower.contains("subscribe")
                || lower.contains("sign up")
                || lower.contains("signup");
            let has_newsletter =
                lower.contains("newsletter") || lower.contains("email") || lower.contains("inbox");

            if has_subscribe && has_newsletter {
                record_removal(removals, debug, "newsletter signup", &text);
                to_remove.push(node_id);
            }
        }
    }

    for id in to_remove {
        dom::remove_node(html, id);
    }
}

/// Rule 13: Author/contact info blocks - near end, with email/phone.
fn remove_author_contact_blocks(
    html: &mut Html,
    main_content: NodeId,
    removals: &mut Vec<Removal>,
    debug: bool,
) {
    let children = dom::child_elements(html, main_content);
    let len = children.len();
    if len < 2 {
        return;
    }

    let mut to_remove = Vec::new();
    let start = len.saturating_sub(4);

    for &child_id in &children[start..] {
        let tag = dom::tag_name(html, child_id);
        if !matches!(tag.as_deref(), Some("div" | "section" | "aside")) {
            continue;
        }

        let text = dom::text_content(html, child_id);
        let lower = text.to_lowercase();
        let word_count = dom::count_words(&text);

        if word_count > 60 {
            continue;
        }

        let has_contact = lower.contains("email:")
            || lower.contains("phone:")
            || lower.contains("contact:")
            || lower.contains("tel:")
            || lower.contains("e-mail:");

        if has_contact {
            record_removal(removals, debug, "author contact block", &text);
            to_remove.push(child_id);
        }
    }

    for id in to_remove {
        dom::remove_node(html, id);
    }
}

/// Rule 14: Author/share widgets - "Author", "Share", "Written by"
/// labels in small containers.
fn remove_author_share_widgets(
    html: &mut Html,
    main_content: NodeId,
    removals: &mut Vec<Removal>,
    debug: bool,
) {
    let tags = ["div", "span", "aside", "section"];
    let mut to_remove = Vec::new();

    for tag in &tags {
        let elements = dom::descendant_elements_by_tag(html, main_content, tag);
        for node_id in elements {
            let text = dom::text_content(html, node_id);
            let word_count = dom::count_words(&text);
            if word_count > 20 {
                continue;
            }

            let trimmed = text.trim();
            let lower = trimmed.to_lowercase();
            let is_share_widget = lower.starts_with("share") || lower.starts_with("share this");
            let is_author_widget = lower.starts_with("author")
                || lower.starts_with("written by")
                || lower.starts_with("posted by");

            // Author-prefixed elements that contain actual names
            // (e.g. "Authors: Name1, Name2") are article metadata,
            // not share/navigation widgets.
            if is_author_widget && looks_like_author_list(&lower) {
                continue;
            }

            let is_widget = is_share_widget || is_author_widget;
            if is_widget && word_count <= 12 {
                record_removal(removals, debug, "author/share widget", &text);
                to_remove.push(node_id);
            }
        }
    }

    for id in to_remove {
        dom::remove_node(html, id);
    }
}

/// Rule 15: Social engagement counters - "9 Likes", "3 Comments".
fn remove_social_counters(
    html: &mut Html,
    main_content: NodeId,
    removals: &mut Vec<Removal>,
    debug: bool,
) {
    let tags = ["span", "div", "p", "li"];
    let mut to_remove = Vec::new();

    for tag in &tags {
        let elements = dom::descendant_elements_by_tag(html, main_content, tag);
        for node_id in elements {
            let text = dom::text_content(html, node_id);
            if SOCIAL_COUNTER_RE.is_match(text.trim()) {
                record_removal(removals, debug, "social counter", &text);
                to_remove.push(node_id);
            }
        }
    }

    for id in to_remove {
        dom::remove_node(html, id);
    }
}

/// Rule 16: Related intro paragraphs - "For more on/about...".
fn remove_related_intro_paragraphs(
    html: &mut Html,
    main_content: NodeId,
    removals: &mut Vec<Removal>,
    debug: bool,
) {
    let paragraphs = dom::descendant_elements_by_tag(html, main_content, "p");
    let mut to_remove = Vec::new();

    for p_id in paragraphs {
        let text = dom::text_content(html, p_id);
        let word_count = dom::count_words(&text);
        if word_count > 25 {
            continue;
        }

        if RELATED_INTRO_RE.is_match(text.trim()) {
            let density = dom::link_density(html, p_id);
            if density > 0.3 {
                record_removal(removals, debug, "related intro paragraph", &text);
                to_remove.push(p_id);
            }
        }
    }

    for id in to_remove {
        dom::remove_node(html, id);
    }
}

/// Rule 17: Related post card grids - divs with 2+ children each
/// containing img + heading/link.
fn remove_related_post_card_grids(
    html: &mut Html,
    main_content: NodeId,
    removals: &mut Vec<Removal>,
    debug: bool,
) {
    let children = dom::child_elements(html, main_content);
    let mut to_remove = Vec::new();

    for &child_id in &children {
        let tag = dom::tag_name(html, child_id);
        if !matches!(tag.as_deref(), Some("div" | "section")) {
            continue;
        }

        let cards = dom::child_elements(html, child_id);
        if cards.len() < 2 {
            continue;
        }

        let card_count = cards
            .iter()
            .filter(|&&card_id| is_card_element(html, card_id))
            .count();

        if card_count >= 2 && card_count == cards.len() {
            let text = dom::text_content(html, child_id);
            record_removal(removals, debug, "related post card grid", &text);
            to_remove.push(child_id);
        }
    }

    for id in to_remove {
        dom::remove_node(html, id);
    }
}

/// Rule 19: Pinned labels - "pinned" text in small elements.
fn remove_pinned_labels(
    html: &mut Html,
    main_content: NodeId,
    removals: &mut Vec<Removal>,
    debug: bool,
) {
    let tags = ["span", "div", "p", "label"];
    let mut to_remove = Vec::new();

    for tag in &tags {
        let elements = dom::descendant_elements_by_tag(html, main_content, tag);
        for node_id in elements {
            let text = dom::text_content(html, node_id);
            if dom::count_words(&text) <= 3 && PINNED_RE.is_match(&text) {
                record_removal(removals, debug, "pinned label", &text);
                to_remove.push(node_id);
            }
        }
    }

    for id in to_remove {
        dom::remove_node(html, id);
    }
}

/// Rule 20: Timezone widgets - "current time in" pattern.
fn remove_timezone_widgets(
    html: &mut Html,
    main_content: NodeId,
    removals: &mut Vec<Removal>,
    debug: bool,
) {
    let tags = ["div", "span", "p", "section"];
    let mut to_remove = Vec::new();

    for tag in &tags {
        let elements = dom::descendant_elements_by_tag(html, main_content, tag);
        for node_id in elements {
            let text = dom::text_content(html, node_id);
            let word_count = dom::count_words(&text);
            if word_count <= 15 && TIMEZONE_RE.is_match(&text) {
                record_removal(removals, debug, "timezone widget", &text);
                to_remove.push(node_id);
            }
        }
    }

    for id in to_remove {
        dom::remove_node(html, id);
    }
}

// ── Helpers ──────────────────────────────────────────────────────

fn next_element_sibling(html: &Html, node_id: NodeId) -> Option<NodeId> {
    let node_ref = html.tree.get(node_id)?;
    let mut sibling = node_ref.next_sibling();
    while let Some(s) = sibling {
        if matches!(s.value(), scraper::Node::Element(_)) {
            return Some(s.id());
        }
        sibling = s.next_sibling();
    }
    None
}

fn record_removal(removals: &mut Vec<Removal>, debug: bool, reason: &str, text: &str) {
    if debug {
        let trimmed = text.split_whitespace().collect::<Vec<_>>().join(" ");
        let preview = match trimmed.char_indices().nth(80) {
            Some((i, _)) => format!("{}...", &trimmed[..i]),
            None => trimmed,
        };
        removals.push(Removal {
            step: "removeByContentPattern".into(),
            selector: None,
            reason: Some(reason.into()),
            text: preview,
        });
    }
}

fn is_heading_tag(tag: Option<&str>) -> bool {
    matches!(tag, Some("h1" | "h2" | "h3" | "h4" | "h5" | "h6"))
}

/// Collect node IDs from first and last N children (boundary elements).
fn collect_boundary_ids(children: &[NodeId], n: usize) -> Vec<NodeId> {
    let len = children.len();
    let mut ids = Vec::new();
    let head_end = len.min(n);
    for &id in &children[..head_end] {
        ids.push(id);
    }
    let tail_start = len.saturating_sub(n);
    for &id in &children[tail_start..] {
        if !ids.contains(&id) {
            ids.push(id);
        }
    }
    ids
}

/// Check if an element looks like a card (has img + heading or link).
fn is_card_element(html: &Html, node_id: NodeId) -> bool {
    let has_img = !dom::descendant_elements_by_tag(html, node_id, "img").is_empty();
    if !has_img {
        return false;
    }

    let has_link = !dom::descendant_elements_by_tag(html, node_id, "a").is_empty();
    let has_heading = ["h1", "h2", "h3", "h4", "h5", "h6"]
        .iter()
        .any(|t| !dom::descendant_elements_by_tag(html, node_id, t).is_empty());

    has_link || has_heading
}
