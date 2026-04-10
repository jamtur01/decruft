use std::collections::HashSet;

use ego_tree::NodeId;
use scraper::{Html, Node};

use crate::dom;
use crate::scorer;
use crate::selectors;
use crate::types::Removal;

/// Remove elements matching exact CSS selectors.
pub fn remove_exact_selectors(
    html: &mut Html,
    main_content: NodeId,
    removals: &mut Vec<Removal>,
    debug: bool,
) {
    for selector_str in selectors::EXACT_SELECTORS {
        let ids = dom::select_ids(html, selector_str);
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

/// Remove elements matching partial class/id patterns.
pub fn remove_partial_selectors(
    html: &mut Html,
    main_content: NodeId,
    removals: &mut Vec<Removal>,
    debug: bool,
) {
    let pattern = selectors::partial_pattern();
    let mut to_remove = Vec::new();

    let all_elements = collect_all_elements(html);
    for node_id in all_elements {
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
                && pattern.is_match(val)
            {
                matched = true;
                break;
            }
        }

        if matched {
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

/// Remove hidden elements (display:none, visibility:hidden, etc.).
pub fn remove_hidden_elements(
    html: &mut Html,
    main_content: NodeId,
    removals: &mut Vec<Removal>,
    debug: bool,
) {
    let mut to_remove = Vec::new();

    let all_elements = collect_all_elements(html);
    for node_id in all_elements {
        if dom::is_ancestor(html, main_content, node_id) {
            continue;
        }

        let Some(node_ref) = html.tree.get(node_id) else {
            continue;
        };
        let Node::Element(el) = node_ref.value() else {
            continue;
        };

        let is_hidden = if let Some(style) = el.attr("style") {
            let s = style.to_lowercase();
            s.contains("display:none")
                || s.contains("display: none")
                || s.contains("visibility:hidden")
                || s.contains("visibility: hidden")
                || s.contains("opacity:0")
                || s.contains("opacity: 0")
        } else {
            false
        };

        let has_hidden_class = if let Some(class) = el.attr("class") {
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
            if has_responsive {
                false
            } else {
                classes
                    .iter()
                    .any(|c| *c == "hidden" || *c == ":hidden")
            }
        } else {
            false
        };

        if is_hidden || has_hidden_class {
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
    }

    for id in to_remove {
        dom::remove_node(html, id);
    }
}

/// Score and remove non-content blocks.
pub fn score_and_remove(
    html: &mut Html,
    main_content: NodeId,
    removals: &mut Vec<Removal>,
    debug: bool,
) {
    let block_tags = [
        "div", "section", "article", "main", "aside", "header",
        "footer", "nav",
    ];

    let mut to_remove = Vec::new();

    let all_elements = collect_all_elements(html);
    for node_id in all_elements {
        if dom::is_ancestor(html, main_content, node_id) {
            continue;
        }
        if node_id == main_content {
            continue;
        }
        if is_inside_pre_or_code(html, node_id) {
            continue;
        }

        let Some(tag) = dom::tag_name(html, node_id) else {
            continue;
        };
        if !block_tags.contains(&tag.as_str()) {
            continue;
        }

        if scorer::is_likely_content(html, node_id) {
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

        let width = el
            .attr("width")
            .and_then(|w| w.parse::<u32>().ok());
        let height = el
            .attr("height")
            .and_then(|h| h.parse::<u32>().ok());

        let is_small = match (width, height) {
            (Some(w), _) if w < 33 => true,
            (_, Some(h)) if h < 33 => true,
            _ => false,
        };

        let is_placeholder = el.attr("src").is_none_or(|s| {
            s.is_empty() || s.starts_with("data:image/gif")
        });

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
    let img_ids =
        dom::descendant_elements_by_tag(html, main_content, "img");
    for id in img_ids {
        dom::remove_node(html, id);
    }
    let picture_ids =
        dom::descendant_elements_by_tag(html, main_content, "picture");
    for id in picture_ids {
        dom::remove_node(html, id);
    }
}

fn collect_all_elements(html: &Html) -> Vec<NodeId> {
    let mut result = Vec::new();
    collect_elements_recursive(html, html.tree.root().id(), &mut result);
    result
}

fn collect_elements_recursive(
    html: &Html,
    node_id: NodeId,
    result: &mut Vec<NodeId>,
) {
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
    if trimmed.len() <= max_len {
        trimmed
    } else {
        format!("{}...", &trimmed[..max_len])
    }
}

/// Collect all meta tags from the document.
pub fn collect_meta_tags(
    html: &Html,
) -> Vec<crate::types::MetaTag> {
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

/// Deduplicate images with same alt text, keeping highest resolution.
pub fn deduplicate_images(html: &mut Html, main_content: NodeId) {
    let img_ids =
        dom::descendant_elements_by_tag(html, main_content, "img");

    let mut seen_alts: HashSet<String> = HashSet::new();
    let mut to_remove = Vec::new();

    for img_id in img_ids {
        let alt = dom::get_attr(html, img_id, "alt")
            .unwrap_or_default();
        if alt.is_empty() {
            continue;
        }
        if seen_alts.contains(&alt) {
            to_remove.push(img_id);
        } else {
            seen_alts.insert(alt);
        }
    }

    for id in to_remove {
        dom::remove_node(html, id);
    }
}
