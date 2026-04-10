use std::collections::HashSet;

use ego_tree::NodeId;
use scraper::{Html, Node};

use crate::dom;
use crate::scorer;
use crate::selectors;
use crate::types::Removal;

/// Run hidden, partial, and scoring cleanup in a single pass over all
/// elements, avoiding the cost of three separate tree walks.
#[expect(clippy::fn_params_excessive_bools)]
pub fn run_combined_cleanup(
    html: &mut Html,
    main_content: NodeId,
    removals: &mut Vec<Removal>,
    debug: bool,
    do_hidden: bool,
    do_partial: bool,
    do_scoring: bool,
) {
    let all_elements = collect_all_elements(html);
    if do_hidden {
        remove_hidden_with_elements(html, main_content, removals, debug, &all_elements);
    }
    if do_partial {
        remove_partial_selectors_with_elements(html, main_content, removals, debug, &all_elements);
    }
    if do_scoring {
        score_and_remove_with_elements(html, main_content, removals, debug, &all_elements);
    }
}

/// Remove elements matching exact CSS selectors.
pub fn remove_exact_selectors(
    html: &mut Html,
    main_content: NodeId,
    removals: &mut Vec<Removal>,
    debug: bool,
) {
    for (selector_str, compiled) in selectors::COMPILED_EXACT_SELECTORS.iter() {
        let ids: Vec<NodeId> = html.select(compiled).map(|el| el.id()).collect();
        for id in ids {
            if dom::is_ancestor(html, main_content, id) {
                continue;
            }
            if debug {
                let text = dom::text_content(html, id);
                let preview = truncate(&text, 80);
                removals.push(Removal {
                    step: "removeBySelector".into(),
                    selector: Some((*selector_str).into()),
                    reason: Some("exact selector match".into()),
                    text: preview,
                });
            }
            dom::remove_node(html, id);
        }
    }
}

fn remove_partial_selectors_with_elements(
    html: &mut Html,
    main_content: NodeId,
    removals: &mut Vec<Removal>,
    debug: bool,
    all_elements: &[NodeId],
) {
    let mut to_remove = Vec::new();

    for &node_id in all_elements {
        if dom::is_ancestor(html, main_content, node_id) {
            continue;
        }
        if is_inside_pre_or_code(html, node_id) {
            continue;
        }

        let Some(node_ref) = html.tree.get(node_id) else {
            continue;
        };
        let Node::Element(el) = node_ref.value() else {
            continue;
        };

        let mut matched = false;
        for attr in selectors::PARTIAL_ATTRIBUTES {
            if let Some(val) = el.attr(attr)
                && selectors::matches_partial(val)
            {
                matched = true;
                break;
            }
        }

        if matched {
            // Protect article metadata (authors, citations, dates,
            // subjects) inside the content root from partial removal.
            if scorer::is_article_metadata(html, node_id)
                && dom::is_ancestor(html, node_id, main_content)
            {
                continue;
            }

            if debug {
                let text = dom::text_content(html, node_id);
                removals.push(Removal {
                    step: "removeBySelector".into(),
                    selector: None,
                    reason: Some("partial selector match".into()),
                    text: truncate(&text, 80),
                });
            }
            to_remove.push(node_id);
        }
    }

    for id in to_remove {
        dom::remove_node(html, id);
    }
}

fn remove_hidden_with_elements(
    html: &mut Html,
    main_content: NodeId,
    removals: &mut Vec<Removal>,
    debug: bool,
    all_elements: &[NodeId],
) {
    let mut to_remove = Vec::new();

    for &node_id in all_elements {
        if dom::is_ancestor(html, main_content, node_id) {
            continue;
        }
        if !is_hidden_element(html, node_id) {
            continue;
        }
        if debug {
            let text = dom::text_content(html, node_id);
            removals.push(Removal {
                step: "removeHiddenElements".into(),
                selector: None,
                reason: Some("hidden element".into()),
                text: truncate(&text, 80),
            });
        }
        to_remove.push(node_id);
    }

    for id in to_remove {
        dom::remove_node(html, id);
    }
}

/// Check whether an element is hidden via inline styles, HTML
/// attributes, or CSS classes. Covers:
/// - inline `display:none`, `visibility:hidden`, `opacity:0`
/// - `hidden` boolean attribute
/// - `aria-hidden="true"` attribute
/// - `inert` attribute
/// - `.hidden`, `:hidden`, `.invisible` classes (with responsive
///   class exclusion for Tailwind breakpoints)
#[must_use]
pub fn is_hidden_element(html: &Html, node_id: NodeId) -> bool {
    let Some(node_ref) = html.tree.get(node_id) else {
        return false;
    };
    let Node::Element(el) = node_ref.value() else {
        return false;
    };

    // Inline style checks
    if let Some(style) = el.attr("style") {
        let s = style.to_lowercase();
        if s.contains("display:none")
            || s.contains("display: none")
            || s.contains("visibility:hidden")
            || s.contains("visibility: hidden")
            || s.contains("opacity:0")
            || s.contains("opacity: 0")
        {
            return true;
        }
    }

    // HTML boolean `hidden` attribute
    if el.attr("hidden").is_some() {
        return true;
    }

    // aria-hidden="true"
    if el.attr("aria-hidden").is_some_and(|v| v == "true") {
        return true;
    }

    // inert attribute
    if el.attr("inert").is_some() {
        return true;
    }

    // Class-based hidden checks (skip if responsive override present)
    if let Some(class) = el.attr("class") {
        let classes: Vec<&str> = class.split_whitespace().collect();
        let has_responsive = classes.iter().any(|c| {
            c.contains("sm:block")
                || c.contains("md:block")
                || c.contains("lg:block")
                || c.contains("xl:block")
                || c.contains("sm:flex")
                || c.contains("md:flex")
                || c.contains("lg:flex")
                || c.contains("xl:flex")
        });
        if !has_responsive
            && classes
                .iter()
                .any(|c| *c == "hidden" || *c == ":hidden" || *c == "invisible")
        {
            return true;
        }
    }

    false
}

fn score_and_remove_with_elements(
    html: &mut Html,
    main_content: NodeId,
    removals: &mut Vec<Removal>,
    debug: bool,
    all_elements: &[NodeId],
) {
    let block_tags: HashSet<&str> = [
        "div", "section", "article", "main", "aside", "header", "footer", "nav",
    ]
    .into_iter()
    .collect();

    // Pre-collect ancestors of main_content to avoid O(depth)
    // is_ancestor checks per element.
    let main_ancestors: HashSet<NodeId> = collect_ancestors(html, main_content);

    let mut to_remove = Vec::new();
    let mut removed_set: HashSet<NodeId> = HashSet::new();

    // Process in reverse (deepest first) so children are scored
    // before parents, reducing redundant subtree walks.
    for &node_id in all_elements.iter().rev() {
        if node_id == main_content || main_ancestors.contains(&node_id) {
            continue;
        }
        // Skip if this element is inside an already-removed ancestor
        if is_descendant_of_set(html, node_id, &removed_set) {
            continue;
        }
        if is_inside_pre_or_code(html, node_id) {
            continue;
        }
        // Skip elements inside footnote-protected containers (references, citations)
        if is_inside_footnote_container(html, node_id) {
            continue;
        }

        let Some(node_ref) = html.tree.get(node_id) else {
            continue;
        };
        let Node::Element(el) = node_ref.value() else {
            continue;
        };
        let tag = el.name.local.as_ref();
        if !block_tags.contains(tag) {
            continue;
        }

        if scorer::is_likely_content(html, node_id) {
            continue;
        }

        // Protect metadata elements (authors, dates, citations) that
        // are close to the content root (direct child or grandchild).
        if scorer::is_article_metadata(html, node_id)
            && is_near_content_root(html, node_id, main_content)
        {
            continue;
        }

        let score = scorer::score_non_content(html, node_id);
        if score < 0.0 {
            if debug {
                let text = dom::text_content(html, node_id);
                removals.push(Removal {
                    step: "scoreAndRemove".into(),
                    selector: None,
                    reason: Some(format!("score: {score:.1}")),
                    text: truncate(&text, 80),
                });
            }
            to_remove.push(node_id);
            removed_set.insert(node_id);
        }
    }

    for id in to_remove {
        dom::remove_node(html, id);
    }
}

/// Remove small images (< 33px in either dimension).
pub fn remove_small_images(html: &mut Html, main_content: NodeId) {
    let img_ids = dom::descendant_elements_by_tag(html, main_content, "img");
    let mut to_remove = Vec::new();

    for img_id in img_ids {
        let Some(node_ref) = html.tree.get(img_id) else {
            continue;
        };
        let Node::Element(el) = node_ref.value() else {
            continue;
        };

        let width = el.attr("width").and_then(|w| w.parse::<u32>().ok());
        let height = el.attr("height").and_then(|h| h.parse::<u32>().ok());

        let is_small = match (width, height) {
            (Some(w), _) if w < 33 => true,
            (_, Some(h)) if h < 33 => true,
            _ => false,
        };

        let is_placeholder = el
            .attr("src")
            .is_none_or(|s| s.is_empty() || s.starts_with("data:image/gif"));

        if is_small || is_placeholder {
            to_remove.push(img_id);
        }
    }

    for id in to_remove {
        dom::remove_node(html, id);
    }
}

/// Remove all images from content.
pub fn remove_all_images(html: &mut Html, main_content: NodeId) {
    let img_ids = dom::descendant_elements_by_tag(html, main_content, "img");
    for id in img_ids {
        dom::remove_node(html, id);
    }
    let picture_ids = dom::descendant_elements_by_tag(html, main_content, "picture");
    for id in picture_ids {
        dom::remove_node(html, id);
    }
}

/// Remove `<header>` elements that don't wrap article content.
///
/// Defuddle uses `header:not(:has(p + p))` to keep headers that
/// contain consecutive paragraphs (content wrappers). Since scraper
/// doesn't support `:has()`, we check manually: skip removal if the
/// header contains two or more consecutive `<p>` siblings.
pub fn remove_header_elements(
    html: &mut Html,
    main_content: NodeId,
    removals: &mut Vec<Removal>,
    debug: bool,
) {
    let ids = dom::select_ids(html, "header");
    for id in ids {
        if dom::is_ancestor(html, main_content, id) {
            continue;
        }
        if has_consecutive_paragraphs(html, id) {
            continue;
        }
        if debug {
            let text = dom::text_content(html, id);
            let preview = truncate(&text, 80);
            removals.push(Removal {
                step: "removeBySelector".into(),
                selector: Some("header".into()),
                reason: Some("header without content paragraphs".into()),
                text: preview,
            });
        }
        dom::remove_node(html, id);
    }
}

/// Check if a node contains two or more consecutive `<p>` children
/// at any depth (searches all descendant containers).
fn has_consecutive_paragraphs(html: &Html, node_id: NodeId) -> bool {
    let Some(node_ref) = html.tree.get(node_id) else {
        return false;
    };

    // Check direct element children for consecutive <p> elements,
    // skipping whitespace text nodes between them.
    let mut prev_was_p = false;
    for child in node_ref.children() {
        let Node::Element(el) = child.value() else {
            continue;
        };
        let is_p = el.name.local.as_ref() == "p";
        if is_p && prev_was_p {
            return true;
        }
        prev_was_p = is_p;
    }

    // Recurse into child elements
    for child in node_ref.children() {
        if matches!(child.value(), Node::Element(_)) && has_consecutive_paragraphs(html, child.id())
        {
            return true;
        }
    }

    false
}

/// Check if any ancestor of `node_id` is in the given set.
fn is_descendant_of_set(html: &Html, node_id: NodeId, set: &HashSet<NodeId>) -> bool {
    let mut current = node_id;
    loop {
        let Some(node_ref) = html.tree.get(current) else {
            return false;
        };
        let Some(parent) = node_ref.parent() else {
            return false;
        };
        if set.contains(&parent.id()) {
            return true;
        }
        current = parent.id();
    }
}

/// Collect all ancestor element IDs of a node (walking up to root).
fn collect_ancestors(html: &Html, node_id: NodeId) -> HashSet<NodeId> {
    let mut ancestors = HashSet::new();
    let mut current = node_id;
    loop {
        let Some(node_ref) = html.tree.get(current) else {
            break;
        };
        let Some(parent) = node_ref.parent() else {
            break;
        };
        if matches!(parent.value(), Node::Element(_)) {
            ancestors.insert(parent.id());
        }
        current = parent.id();
    }
    ancestors
}

fn collect_all_elements(html: &Html) -> Vec<NodeId> {
    let mut result = Vec::new();
    collect_elements_recursive(html, html.tree.root().id(), &mut result);
    result
}

fn collect_elements_recursive(html: &Html, node_id: NodeId, result: &mut Vec<NodeId>) {
    let Some(node_ref) = html.tree.get(node_id) else {
        return;
    };
    for child in node_ref.children() {
        if matches!(child.value(), Node::Element(_)) {
            result.push(child.id());
        }
        collect_elements_recursive(html, child.id(), result);
    }
}

/// Check if `node_id` is a direct child or grandchild of
/// `content_root` (within 2 levels of nesting).
fn is_near_content_root(html: &Html, node_id: NodeId, content_root: NodeId) -> bool {
    let Some(parent) = dom::parent_element(html, node_id) else {
        return false;
    };
    if parent == content_root {
        return true;
    }
    let Some(grandparent) = dom::parent_element(html, parent) else {
        return false;
    };
    grandparent == content_root
}

/// Check if an element is inside a footnote/reference container.
/// Checks both data-decruft-footnote (pre-standardization protection)
/// and canonical footnote IDs (post-standardization).
fn is_inside_footnote_container(html: &Html, node_id: NodeId) -> bool {
    let mut current = node_id;
    loop {
        // Check data-decruft-footnote attribute (set during protection pass)
        if dom::get_attr(html, current, "data-decruft-footnote").is_some() {
            return true;
        }
        // Check canonical footnote structure (set during standardization)
        if let Some(id) = dom::get_attr(html, current, "id")
            && (id == "footnotes" || id.starts_with("fn:"))
        {
            return true;
        }
        if let Some(class) = dom::get_attr(html, current, "class")
            && (class.contains("footnote")
                || class.contains("reflist")
                || class.contains("references"))
        {
            return true;
        }
        let Some(parent_id) = dom::parent_element(html, current) else {
            return false;
        };
        current = parent_id;
    }
}

fn is_inside_pre_or_code(html: &Html, node_id: NodeId) -> bool {
    let mut current = node_id;
    loop {
        let Some(node_ref) = html.tree.get(current) else {
            return false;
        };
        if let Node::Element(el) = node_ref.value() {
            let tag = el.name.local.as_ref();
            if tag == "pre" || tag == "code" {
                return true;
            }
        }
        let Some(parent) = node_ref.parent() else {
            return false;
        };
        current = parent.id();
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    let trimmed = s.split_whitespace().collect::<Vec<_>>().join(" ");
    match trimmed.char_indices().nth(max_len) {
        Some((i, _)) => format!("{}...", &trimmed[..i]),
        None => trimmed,
    }
}

/// Collect all meta tags from the document.
pub fn collect_meta_tags(html: &Html) -> Vec<crate::types::MetaTag> {
    let mut tags = Vec::new();
    let ids = dom::select_ids(html, "meta");
    for id in ids {
        let Some(node_ref) = html.tree.get(id) else {
            continue;
        };
        let Node::Element(el) = node_ref.value() else {
            continue;
        };
        tags.push(crate::types::MetaTag {
            name: el.attr("name").map(String::from),
            property: el.attr("property").map(String::from),
            content: el.attr("content").map(String::from),
        });
    }
    tags
}

/// Deduplicate images with same alt text, keeping highest quality.
///
/// When duplicates are found, compares width/height attributes,
/// srcset presence, and src URL length to pick the better image.
pub fn deduplicate_images(html: &mut Html, main_content: NodeId) {
    use std::collections::HashMap;

    let img_ids = dom::descendant_elements_by_tag(html, main_content, "img");

    let mut seen_alts: HashMap<String, NodeId> = HashMap::new();
    let mut to_remove = Vec::new();

    for img_id in img_ids {
        let alt = dom::get_attr(html, img_id, "alt").unwrap_or_default();
        if alt.is_empty() {
            continue;
        }
        if let Some(&existing_id) = seen_alts.get(&alt) {
            let (keep, discard) = pick_better_image(html, existing_id, img_id);
            to_remove.push(discard);
            seen_alts.insert(alt, keep);
        } else {
            seen_alts.insert(alt, img_id);
        }
    }

    for id in to_remove {
        dom::remove_node(html, id);
    }
}

/// Compare two images and return (keep, discard) based on quality
/// signals: dimensions, srcset, and URL length.
fn pick_better_image(html: &Html, a: NodeId, b: NodeId) -> (NodeId, NodeId) {
    let score_a = image_quality_score(html, a);
    let score_b = image_quality_score(html, b);
    if score_b > score_a { (b, a) } else { (a, b) }
}

/// Compute a simple quality score for an image based on:
/// - width * height (if attributes present)
/// - srcset bonus (indicates responsive/multiple resolutions)
/// - src URL length (longer URLs often have quality params)
fn image_quality_score(html: &Html, node_id: NodeId) -> u64 {
    let width = dom::get_attr(html, node_id, "width")
        .and_then(|w| w.parse::<u64>().ok())
        .unwrap_or(0);
    let height = dom::get_attr(html, node_id, "height")
        .and_then(|h| h.parse::<u64>().ok())
        .unwrap_or(0);

    let dimension_score = if width > 0 && height > 0 {
        width * height
    } else {
        width + height
    };

    let srcset_bonus: u64 = if dom::get_attr(html, node_id, "srcset").is_some() {
        10_000
    } else {
        0
    };

    let url_len: u64 = dom::get_attr(html, node_id, "src").map_or(0, |s| s.len() as u64);

    dimension_score + srcset_bonus + url_len
}
