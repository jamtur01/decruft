use ego_tree::NodeId;
use scraper::{Html, Node};

use crate::dom;

/// Allowed attributes to preserve on elements.
const ALLOWED_ATTRIBUTES: &[&str] = &[
    "accent",
    "accentunder",
    "align",
    "alt",
    "allow",
    "allowfullscreen",
    "aria-label",
    "checked",
    "class",
    "colspan",
    "columnalign",
    "columnlines",
    "columnspacing",
    "columnspan",
    "controls",
    "data-callout",
    "data-callout-fold",
    "data-callout-title",
    "data-lang",
    "data-latex",
    "data-mjx-texclass",
    "data-src",
    "data-srcset",
    "depth",
    "dir",
    "display",
    "displaystyle",
    "fence",
    "frame",
    "frameborder",
    "framespacing",
    "headers",
    "height",
    "href",
    "id",
    "kind",
    "label",
    "lang",
    "linethickness",
    "lspace",
    "mathsize",
    "mathvariant",
    "maxsize",
    "minsize",
    "movablelimits",
    "notation",
    "poster",
    "role",
    "rowalign",
    "rowlines",
    "rowspacing",
    "rowspan",
    "rspace",
    "scriptlevel",
    "separator",
    "src",
    "srclang",
    "srcset",
    "stretchy",
    "symmetric",
    "title",
    "type",
    "voffset",
    "width",
    "xmlns",
];

/// Elements that are allowed to be empty.
const ALLOWED_EMPTY: &[&str] = &[
    "area", "audio", "base", "br", "circle", "col", "defs", "ellipse", "embed", "figure", "g",
    "hr", "iframe", "img", "input", "line", "link", "mask", "meta", "object", "param", "path",
    "pattern", "picture", "polygon", "polyline", "rect", "source", "stop", "svg", "td", "th",
    "track", "use", "video", "wbr",
];

/// Unsafe elements to strip entirely.
const UNSAFE_ELEMENTS: &[&str] = &["frame", "frameset", "object", "embed", "applet", "base"];

/// URL attributes that should not contain script URIs.
const URL_ATTRS: &[&str] = &["href", "src", "action", "formaction"];

/// Standardize content: clean attributes, remove empty elements,
/// fix headings, unwrap wrapper divs, and strip `<wbr>` tags.
pub fn standardize_content(html: &mut Html, main_content: NodeId) {
    remove_wbr_elements(html, main_content);
    clean_attributes(html, main_content);
    remove_empty_elements(html, main_content);
    normalize_headings(html, main_content);
    unwrap_wrapper_divs(html, main_content);
}

/// Remove all `<wbr>` elements (word-break hints) without inserting
/// any whitespace.
fn remove_wbr_elements(html: &mut Html, main_content: NodeId) {
    let wbrs = dom::descendant_elements_by_tag(html, main_content, "wbr");
    for id in wbrs {
        dom::remove_node(html, id);
    }
}

/// Remove non-allowed attributes from all elements under the given
/// root. Public alias for use by the extractor sanitization path.
pub fn clean_attributes_on(html: &mut Html, main_content: NodeId) {
    clean_attributes(html, main_content);
}

/// Remove non-allowed attributes from all elements.
fn clean_attributes(html: &mut Html, main_content: NodeId) {
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
            "svg"
                | "path"
                | "circle"
                | "rect"
                | "line"
                | "polygon"
                | "polyline"
                | "g"
                | "defs"
                | "use"
                | "mask"
                | "ellipse"
                | "stop"
                | "pattern"
                | "text"
                | "tspan"
                | "clippath"
                | "lineargradient"
                | "radialgradient"
                | "filter"
                | "fegaussianblur"
                | "feoffset"
                | "feblend"
                | "marker"
                | "symbol"
                | "image"
                | "foreignobject"
                | "desc"
                | "metadata"
                | "style"
        ) || is_inside_svg(html, node_id);

        if is_svg_related {
            continue;
        }

        let Some(mut node_mut) = html.tree.get_mut(node_id) else {
            continue;
        };
        let Node::Element(el) = node_mut.value() else {
            continue;
        };
        el.attrs
            .retain(|(name, _)| ALLOWED_ATTRIBUTES.contains(&name.local.as_ref()));
    }
}

/// Remove empty elements (no text, no children, not in allowed list).
/// Also removes `<p>` elements whose only children are `<br>` tags
/// (visual spacers with no content).
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
            continue;
        }

        // <p> with only <br> children (visual spacers)
        let only_brs = has_only_br_children(html, node_id);
        if tag == "p" && text.trim().is_empty() && only_brs {
            to_remove.push(node_id);
        }
    }

    for id in to_remove {
        dom::remove_node(html, id);
    }
}

/// Check if a node's only element children are `<br>` tags.
fn has_only_br_children(html: &Html, node_id: NodeId) -> bool {
    let Some(node_ref) = html.tree.get(node_id) else {
        return false;
    };
    let mut has_element_child = false;
    for child in node_ref.children() {
        if let Node::Element(el) = child.value() {
            if el.name.local.as_ref() != "br" {
                return false;
            }
            has_element_child = true;
        }
    }
    has_element_child
}

/// Normalize heading levels: ensure exactly one h1, demote extras.
fn normalize_headings(html: &mut Html, main_content: NodeId) {
    let h1s = dom::descendant_elements_by_tag(html, main_content, "h1");
    if h1s.len() <= 1 {
        return;
    }

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

/// Unwrap non-semantic wrapper divs (single-child divs containing a
/// block element with no extra text). Moves children before the
/// wrapper, then removes the empty wrapper.
fn unwrap_wrapper_divs(html: &mut Html, main_content: NodeId) {
    let block_tags = [
        "article",
        "section",
        "div",
        "main",
        "p",
        "blockquote",
        "figure",
        "table",
        "ul",
        "ol",
        "dl",
        "h1",
        "h2",
        "h3",
        "h4",
        "h5",
        "h6",
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

        // Preserve canonical footnote structure
        let el_id = dom::get_attr(html, node_id, "id").unwrap_or_default();
        if el_id == "footnotes" || el_id.starts_with("fn:") {
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

    // Move each wrapper's children before the wrapper, then remove it.
    // Process in reverse so inner wrappers are handled first.
    for &wrapper_id in to_unwrap.iter().rev() {
        let child_ids = collect_child_ids(html, wrapper_id);
        for child_id in child_ids {
            let Some(mut wrapper_mut) = html.tree.get_mut(wrapper_id) else {
                break;
            };
            wrapper_mut.insert_id_before(child_id);
        }
        dom::remove_node(html, wrapper_id);
    }
}

/// Collect all direct child node IDs (elements and text nodes).
fn collect_child_ids(html: &Html, node_id: NodeId) -> Vec<NodeId> {
    let Some(node_ref) = html.tree.get(node_id) else {
        return Vec::new();
    };
    node_ref.children().map(|c| c.id()).collect()
}

/// Strip unsafe elements and dangerous attributes from the document.
/// Removes frames, embeds, applets, event handlers, and script URIs.
pub fn strip_unsafe_elements(html: &mut Html) {
    remove_unsafe_tags(html);
    remove_dangerous_attributes(html);
}

/// Remove elements that should never appear in sanitized output.
fn remove_unsafe_tags(html: &mut Html) {
    let mut to_remove = Vec::new();
    for tag in UNSAFE_ELEMENTS {
        let ids = dom::select_ids(html, tag);
        to_remove.extend(ids);
    }
    for id in to_remove {
        dom::remove_node(html, id);
    }
}

/// Remove event handler attributes, srcdoc, and script URIs.
fn remove_dangerous_attributes(html: &mut Html) {
    let all = dom::all_descendant_elements(html, html.tree.root().id());
    for node_id in all {
        let is_svg_child = is_inside_svg(html, node_id);
        let Some(mut node_mut) = html.tree.get_mut(node_id) else {
            continue;
        };
        let Node::Element(el) = node_mut.value() else {
            continue;
        };

        // Preserve <style> inside <svg>
        if is_svg_child && el.name.local.as_ref() == "style" {
            continue;
        }

        el.attrs.retain(|(name, value)| {
            let n = name.local.as_ref();

            // Remove on* event handlers
            if n.starts_with("on") {
                return false;
            }

            // Remove srcdoc
            if n == "srcdoc" {
                return false;
            }

            // Check URL attrs for script URIs
            if URL_ATTRS.contains(&n) {
                let lower = value.to_ascii_lowercase();
                let trimmed = lower.trim();
                if trimmed.starts_with("javascript:") || trimmed.starts_with("data:text/html") {
                    return false;
                }
            }

            true
        });
    }
}

/// Check whether a node is inside an `<svg>` ancestor.
fn is_inside_svg(html: &Html, node_id: NodeId) -> bool {
    let mut current = node_id;
    loop {
        let Some(node_ref) = html.tree.get(current) else {
            return false;
        };
        let Some(parent) = node_ref.parent() else {
            return false;
        };
        if let Node::Element(el) = parent.value()
            && el.name.local.as_ref() == "svg"
        {
            return true;
        }
        current = parent.id();
    }
}

/// Resolve relative URLs to absolute using the base URL.
/// Falls back to `<base href>` if `base_url` is empty or invalid.
pub fn resolve_urls(html: &mut Html, main_content: NodeId, base_url: &str) {
    let base = resolve_base_url(html, base_url);
    let Some(base) = base else {
        return;
    };

    let attrs_to_resolve = [
        ("a", "href"),
        ("img", "src"),
        ("img", "srcset"),
        ("video", "poster"),
        ("source", "src"),
        ("iframe", "src"),
    ];

    for (tag, attr) in &attrs_to_resolve {
        let elements = dom::descendant_elements_by_tag(html, main_content, tag);
        for node_id in elements {
            resolve_single_attr(html, node_id, &base, attr);
        }
    }
}

/// Determine the base URL: prefer the provided URL, fall back to
/// `<base href>` in the document.
fn resolve_base_url(html: &Html, base_url: &str) -> Option<url::Url> {
    if let Ok(parsed) = url::Url::parse(base_url) {
        return Some(parsed);
    }

    // Fall back to <base href> in the document
    let base_ids = dom::select_ids(html, "base[href]");
    for id in base_ids {
        if let Some(href) = dom::get_attr(html, id, "href")
            && let Ok(parsed) = url::Url::parse(&href)
        {
            return Some(parsed);
        }
    }

    None
}

/// Resolve a single URL attribute on a node.
fn resolve_single_attr(html: &mut Html, node_id: NodeId, base: &url::Url, attr: &str) {
    let Some(val) = dom::get_attr(html, node_id, attr) else {
        return;
    };
    if val.starts_with("http://") || val.starts_with("https://") || val.starts_with("//") {
        return;
    }
    if attr == "srcset" {
        resolve_srcset(html, node_id, base);
        return;
    }
    let Ok(resolved) = base.join(&val) else {
        return;
    };
    dom::set_attr(html, node_id, attr, resolved.as_ref());
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
        let descriptor: String = tokens.collect::<Vec<_>>().join(" ");
        let resolved = if url_part.starts_with("http://") || url_part.starts_with("https://") {
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
    dom::set_attr(html, node_id, "srcset", &new_val);
}

/// Parse an HTML string, strip unsafe elements and attributes, then
/// serialize back. Used to sanitize content that bypasses the normal
/// DOM pipeline (e.g. schema.org fallback text).
#[must_use]
pub fn sanitize_html_string(html_str: &str) -> String {
    let mut html = Html::parse_fragment(html_str);
    strip_unsafe_elements(&mut html);
    dom::inner_html(&html, html.tree.root().id())
}
