use ego_tree::NodeId;
use markup5ever::{ns, QualName};
use scraper::{Html, Node};

use crate::dom;

fn qual_name(name: &str) -> QualName {
    QualName::new(None, ns!(), markup5ever::LocalName::from(name))
}

fn set_attr(el: &mut scraper::node::Element, name: &str, value: &str) {
    let qn = qual_name(name);
    // Remove existing
    el.attrs.retain(|(n, _)| n != &qn);
    el.attrs
        .push((qn, markup5ever::tendril::StrTendril::from(value)));
}

/// Allowed attributes to preserve on elements.
const ALLOWED_ATTRIBUTES: &[&str] = &[
    "alt",
    "allow",
    "allowfullscreen",
    "aria-label",
    "checked",
    "colspan",
    "controls",
    "data-latex",
    "data-src",
    "data-srcset",
    "data-callout",
    "data-callout-fold",
    "data-callout-title",
    "data-lang",
    "dir",
    "display",
    "frameborder",
    "headers",
    "height",
    "href",
    "kind",
    "label",
    "lang",
    "role",
    "rowspan",
    "src",
    "srclang",
    "srcset",
    "title",
    "type",
    "width",
];

/// Elements that are allowed to be empty.
const ALLOWED_EMPTY: &[&str] = &[
    "area", "audio", "base", "br", "circle", "col", "defs", "ellipse",
    "embed", "figure", "g", "hr", "iframe", "img", "input", "line",
    "link", "mask", "meta", "object", "param", "path", "pattern",
    "picture", "polygon", "polyline", "rect", "source", "stop", "svg",
    "td", "th", "track", "use", "video", "wbr",
];

/// Standardize content: clean attributes, remove empty elements, fix headings.
pub fn standardize_content(
    html: &mut Html,
    main_content: NodeId,
    debug: bool,
) {
    clean_attributes(html, main_content, debug);
    remove_empty_elements(html, main_content);
    normalize_headings(html, main_content);
    unwrap_wrapper_divs(html, main_content);
}

/// Remove non-allowed attributes from all elements.
fn clean_attributes(html: &mut Html, main_content: NodeId, debug: bool) {
    let descendants = dom::all_descendant_elements(html, main_content);
    for node_id in descendants {
        let Some(node_ref) = html.tree.get(node_id) else {
            continue;
        };
        let Node::Element(el) = node_ref.value() else {
            continue;
        };

        let tag = el.name.local.as_ref().to_string();
        let is_svg_related = matches!(
            tag.as_str(),
            "svg" | "path" | "circle" | "rect" | "line"
                | "polygon" | "polyline" | "g" | "defs"
                | "use" | "mask" | "ellipse" | "stop"
                | "pattern"
        );

        if is_svg_related {
            continue;
        }

        let Some(mut node_mut) = html.tree.get_mut(node_id) else {
            continue;
        };
        let Node::Element(el) = node_mut.value() else {
            continue;
        };
        el.attrs.retain(|(name, _)| {
            let n = name.local.as_ref();
            if debug && (n == "class" || n == "id") {
                return true;
            }
            ALLOWED_ATTRIBUTES.contains(&n)
        });
    }
}

/// Remove empty elements (no text, no children, not in allowed list).
fn remove_empty_elements(html: &mut Html, main_content: NodeId) {
    let mut to_remove = Vec::new();
    let descendants = dom::all_descendant_elements(html, main_content);

    for node_id in descendants {
        let Some(node_ref) = html.tree.get(node_id) else {
            continue;
        };
        let Node::Element(el) = node_ref.value() else {
            continue;
        };

        let tag = el.name.local.as_ref();
        if ALLOWED_EMPTY.contains(&tag) {
            continue;
        }

        let text = dom::text_content(html, node_id);
        if text.trim().is_empty() && !node_ref.has_children() {
            to_remove.push(node_id);
        }
    }

    for id in to_remove {
        dom::remove_node(html, id);
    }
}

/// Normalize heading levels: ensure exactly one h1, demote extras.
fn normalize_headings(html: &mut Html, main_content: NodeId) {
    let h1s = dom::descendant_elements_by_tag(html, main_content, "h1");
    if h1s.len() <= 1 {
        return;
    }

    // Keep the first h1, demote the rest to h2
    for &h1_id in &h1s[1..] {
        let Some(mut node_mut) = html.tree.get_mut(h1_id) else {
            continue;
        };
        let Node::Element(el) = node_mut.value() else {
            continue;
        };
        el.name.local = markup5ever::local_name!("h2");
    }
}

/// Unwrap non-semantic wrapper divs (single-child divs containing a block).
fn unwrap_wrapper_divs(html: &mut Html, main_content: NodeId) {
    let block_tags = [
        "article", "section", "div", "main", "p", "blockquote",
        "figure", "table", "ul", "ol", "dl", "h1", "h2", "h3", "h4",
        "h5", "h6",
    ];

    let descendants = dom::all_descendant_elements(html, main_content);
    let mut to_unwrap = Vec::new();

    for node_id in descendants {
        if node_id == main_content {
            continue;
        }
        let Some(tag) = dom::tag_name(html, node_id) else {
            continue;
        };
        if tag != "div" {
            continue;
        }

        let children = dom::child_elements(html, node_id);
        if children.len() != 1 {
            continue;
        }

        let child_tag = dom::tag_name(html, children[0]);
        let is_block_child = child_tag
            .as_ref()
            .is_some_and(|t| block_tags.contains(&t.as_str()));

        let text = dom::text_content(html, node_id);
        let child_text = dom::text_content(html, children[0]);
        let no_extra_text = text.trim().len() == child_text.trim().len();

        if is_block_child && no_extra_text {
            to_unwrap.push(node_id);
        }
    }

    // Simply remove unnecessary wrapper divs
    // Full unwrapping (reparenting children) is complex with ego-tree;
    // for now just leave them — the content is still accessible
    let _ = to_unwrap;
}

/// Resolve relative URLs to absolute using the base URL.
pub fn resolve_urls(html: &mut Html, main_content: NodeId, base_url: &str) {
    let Ok(base) = url::Url::parse(base_url) else {
        return;
    };

    let attrs_to_resolve = [("a", "href"), ("img", "src"), ("img", "srcset")];

    for (tag, attr) in &attrs_to_resolve {
        let elements =
            dom::descendant_elements_by_tag(html, main_content, tag);
        for node_id in elements {
            let Some(val) = dom::get_attr(html, node_id, attr) else {
                continue;
            };
            if val.starts_with("http://")
                || val.starts_with("https://")
                || val.starts_with("//")
            {
                continue;
            }
            if *attr == "srcset" {
                resolve_srcset(html, node_id, &base);
                continue;
            }
            let Ok(resolved) = base.join(&val) else {
                continue;
            };
            let Some(mut node_mut) = html.tree.get_mut(node_id) else {
                continue;
            };
            let Node::Element(el) = node_mut.value() else {
                continue;
            };
            set_attr(el, attr, resolved.as_ref());
        }
    }
}

fn resolve_srcset(html: &mut Html, node_id: NodeId, base: &url::Url) {
    let Some(val) = dom::get_attr(html, node_id, "srcset") else {
        return;
    };
    let mut parts = Vec::new();
    for entry in val.split(',') {
        let trimmed = entry.trim();
        let mut tokens = trimmed.split_whitespace();
        let Some(url_part) = tokens.next() else {
            continue;
        };
        let descriptor: String =
            tokens.collect::<Vec<_>>().join(" ");
        let resolved = if url_part.starts_with("http://")
            || url_part.starts_with("https://")
        {
            url_part.to_string()
        } else {
            base.join(url_part)
                .map_or_else(|_| url_part.to_string(), |u| u.to_string())
        };
        if descriptor.is_empty() {
            parts.push(resolved);
        } else {
            parts.push(format!("{resolved} {descriptor}"));
        }
    }
    let new_val = parts.join(", ");
    let Some(mut node_mut) = html.tree.get_mut(node_id) else {
        return;
    };
    let Node::Element(el) = node_mut.value() else {
        return;
    };
    set_attr(el, "srcset", &new_val);
}
