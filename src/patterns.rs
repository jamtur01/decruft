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
        r"(?i)^(related\s*(posts|articles|stories)?|you might also like|recommended|more from|subscribe|follow us|share this|newsletter)",
    )
    .expect("valid regex")
});

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
        let preview = if trimmed.len() > 80 {
            format!("{}...", &trimmed[..80])
        } else {
            trimmed
        };
        removals.push(Removal {
            step: "removeByContentPattern".into(),
            selector: None,
            reason: Some(reason.into()),
            text: preview,
        });
    }
}
