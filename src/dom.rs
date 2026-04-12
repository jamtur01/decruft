use std::fmt::Write;

use ego_tree::NodeId;
use scraper::{Html, Node, Selector};

/// Get the text content of a node and its descendants.
#[must_use]
pub fn text_content(html: &Html, node_id: NodeId) -> String {
    let mut text = String::new();
    collect_text(html, node_id, &mut text);
    text
}

fn collect_text(html: &Html, node_id: NodeId, buf: &mut String) {
    let node_ref = html.tree.get(node_id);
    let Some(node_ref) = node_ref else { return };
    match node_ref.value() {
        Node::Text(t) => buf.push_str(t),
        Node::Element(_) => {
            for child in node_ref.children() {
                collect_text(html, child.id(), buf);
            }
        }
        _ => {}
    }
}

/// Get the outer HTML of a node.
#[must_use]
pub fn outer_html(html: &Html, node_id: NodeId) -> String {
    let node_ref = html.tree.get(node_id);
    let Some(node_ref) = node_ref else {
        return String::new();
    };
    serialize_node(html, node_id, node_ref.value())
}

fn serialize_node(html: &Html, node_id: NodeId, node: &Node) -> String {
    match node {
        Node::Document => {
            let mut out = String::new();
            let Some(node_ref) = html.tree.get(node_id) else {
                return out;
            };
            for child in node_ref.children() {
                out.push_str(&serialize_node(html, child.id(), child.value()));
            }
            out
        }
        Node::Text(t) => text_escape(t),
        Node::Element(el) => {
            let tag = el.name.local.as_ref();
            let mut out = format!("<{tag}");
            for (name, value) in el.attrs() {
                let _ = write!(out, " {name}=\"{}\"", attr_escape(value));
            }

            if is_void_element(tag) {
                out.push('>');
                return out;
            }
            out.push('>');

            let Some(node_ref) = html.tree.get(node_id) else {
                return out;
            };
            for child in node_ref.children() {
                out.push_str(&serialize_node(html, child.id(), child.value()));
            }
            let _ = write!(out, "</{tag}>");
            out
        }
        Node::Comment(c) => format!("<!--{}-->", c.comment),
        _ => String::new(),
    }
}

/// Get the inner HTML of a node.
#[must_use]
pub fn inner_html(html: &Html, node_id: NodeId) -> String {
    let Some(node_ref) = html.tree.get(node_id) else {
        return String::new();
    };
    let mut out = String::new();
    for child in node_ref.children() {
        out.push_str(&serialize_node(html, child.id(), child.value()));
    }
    out
}

fn text_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn attr_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn is_void_element(tag: &str) -> bool {
    matches!(
        tag,
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

/// Remove a node from the tree by detaching it.
pub fn remove_node(html: &mut Html, node_id: NodeId) {
    if let Some(mut node) = html.tree.get_mut(node_id) {
        node.detach();
    }
}

/// Set an attribute on an element node.
pub fn set_attr(html: &mut Html, node_id: NodeId, name: &str, value: &str) {
    let Some(mut node_mut) = html.tree.get_mut(node_id) else {
        return;
    };
    let Node::Element(el) = node_mut.value() else {
        return;
    };
    let qn =
        markup5ever::QualName::new(None, markup5ever::ns!(), markup5ever::LocalName::from(name));
    el.attrs.retain(|(n, _)| n != &qn);
    el.attrs
        .push((qn, markup5ever::tendril::StrTendril::from(value)));
}

/// Select element IDs matching a CSS selector that are descendants
/// of `ancestor_id`.
#[must_use]
pub fn select_within(html: &Html, ancestor_id: NodeId, selector_str: &str) -> Vec<NodeId> {
    let Ok(sel) = Selector::parse(selector_str) else {
        return Vec::new();
    };
    html.select(&sel)
        .filter(|el| is_ancestor(html, el.id(), ancestor_id))
        .map(|el| el.id())
        .collect()
}

/// Get an attribute value from an element node.
pub fn get_attr(html: &Html, node_id: NodeId, attr: &str) -> Option<String> {
    let node_ref = html.tree.get(node_id)?;
    if let Node::Element(el) = node_ref.value() {
        el.attr(attr).map(String::from)
    } else {
        None
    }
}

/// Check if a node is an element with the given tag name.
#[must_use]
pub fn is_tag(html: &Html, node_id: NodeId, tag: &str) -> bool {
    let Some(node_ref) = html.tree.get(node_id) else {
        return false;
    };
    if let Node::Element(el) = node_ref.value() {
        el.name.local.as_ref() == tag
    } else {
        false
    }
}

/// Get the tag name of an element node.
#[must_use]
pub fn tag_name(html: &Html, node_id: NodeId) -> Option<String> {
    let node_ref = html.tree.get(node_id)?;
    if let Node::Element(el) = node_ref.value() {
        Some(el.name.local.as_ref().to_string())
    } else {
        None
    }
}

/// Get all element node IDs matching a CSS selector.
#[must_use]
pub fn select_ids(html: &Html, selector_str: &str) -> Vec<NodeId> {
    let Ok(sel) = Selector::parse(selector_str) else {
        return Vec::new();
    };
    html.select(&sel).map(|el| el.id()).collect()
}

/// Count words in text, with CJK awareness.
#[must_use]
pub fn count_words(text: &str) -> usize {
    let mut count = 0usize;
    let mut in_word = false;

    for ch in text.chars() {
        if is_cjk(ch) {
            count += 1;
            in_word = false;
        } else if ch.is_whitespace() {
            in_word = false;
        } else if !in_word {
            in_word = true;
            count += 1;
        }
    }
    count
}

fn is_cjk(ch: char) -> bool {
    let c = ch as u32;
    // Hiragana, Katakana, CJK Extension A, CJK Unified, CJK Compat, Hangul
    (0x3040..=0x309F).contains(&c)
        || (0x30A0..=0x30FF).contains(&c)
        || (0x3400..=0x4DBF).contains(&c)
        || (0x4E00..=0x9FFF).contains(&c)
        || (0xF900..=0xFAFF).contains(&c)
        || (0xAC00..=0xD7AF).contains(&c)
}

/// Count words in HTML by stripping tags.
#[must_use]
pub fn count_words_html(html_str: &str) -> usize {
    let text = strip_tags(html_str);
    let decoded = decode_entities(&text);
    count_words(&decoded)
}

/// Strip HTML tags and decode common HTML entities, producing plain
/// text suitable for display.
#[must_use]
pub fn strip_html_tags(html: &str) -> String {
    let stripped = strip_tags(html);
    decode_entities(&stripped)
}

fn strip_tags(html: &str) -> String {
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

fn decode_entities(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch != '&' {
            result.push(ch);
            continue;
        }
        // Collect entity up to ';' (max 10 chars to avoid runaway)
        let mut entity = String::new();
        let mut found_semi = false;
        for _ in 0..12 {
            match chars.peek() {
                Some(&';') => {
                    chars.next();
                    found_semi = true;
                    break;
                }
                Some(&c) => {
                    entity.push(c);
                    chars.next();
                }
                None => break,
            }
        }
        if !found_semi {
            // Not a valid entity — emit as-is
            result.push('&');
            result.push_str(&entity);
            continue;
        }
        if let Some(decoded) = decode_named_entity(&entity) {
            result.push(decoded);
        } else {
            // Unrecognized — emit verbatim
            result.push('&');
            result.push_str(&entity);
            result.push(';');
        }
    }
    result
}

/// Decode a named or numeric HTML entity (without the & and ;).
fn decode_named_entity(entity: &str) -> Option<char> {
    // Numeric references
    if let Some(hex) = entity
        .strip_prefix("#x")
        .or_else(|| entity.strip_prefix("#X"))
    {
        return u32::from_str_radix(hex, 16).ok().and_then(char::from_u32);
    }
    if let Some(dec) = entity.strip_prefix('#') {
        return dec.parse::<u32>().ok().and_then(char::from_u32);
    }
    // Named entities (most common)
    match entity {
        "nbsp" => Some('\u{00A0}'),
        "amp" => Some('&'),
        "lt" => Some('<'),
        "gt" => Some('>'),
        "quot" => Some('"'),
        "apos" => Some('\''),
        "mdash" => Some('\u{2014}'),
        "ndash" => Some('\u{2013}'),
        "lsquo" => Some('\u{2018}'),
        "rsquo" => Some('\u{2019}'),
        "ldquo" => Some('\u{201C}'),
        "rdquo" => Some('\u{201D}'),
        "bull" => Some('\u{2022}'),
        "hellip" => Some('\u{2026}'),
        "copy" => Some('\u{00A9}'),
        "reg" => Some('\u{00AE}'),
        "trade" => Some('\u{2122}'),
        "eacute" => Some('\u{00E9}'),
        "egrave" => Some('\u{00E8}'),
        "agrave" => Some('\u{00E0}'),
        "uuml" => Some('\u{00FC}'),
        "ouml" => Some('\u{00F6}'),
        "auml" => Some('\u{00E4}'),
        "ccedil" => Some('\u{00E7}'),
        "ntilde" => Some('\u{00F1}'),
        "szlig" => Some('\u{00DF}'),
        "laquo" => Some('\u{00AB}'),
        "raquo" => Some('\u{00BB}'),
        "sect" => Some('\u{00A7}'),
        "deg" => Some('\u{00B0}'),
        "times" => Some('\u{00D7}'),
        "divide" => Some('\u{00F7}'),
        "minus" => Some('\u{2212}'),
        "euro" => Some('\u{20AC}'),
        "pound" => Some('\u{00A3}'),
        "yen" => Some('\u{00A5}'),
        "cent" => Some('\u{00A2}'),
        _ => None,
    }
}

/// Escape a string for safe use inside an HTML attribute value.
#[must_use]
pub fn html_attr_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Escape text for safe use in HTML content.
#[must_use]
pub fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Check if an element has a specific CSS class.
#[must_use]
pub fn has_class(html: &Html, node_id: NodeId, class: &str) -> bool {
    let Some(node_ref) = html.tree.get(node_id) else {
        return false;
    };
    let Node::Element(el) = node_ref.value() else {
        return false;
    };
    el.attr("class")
        .is_some_and(|c| c.split_whitespace().any(|cls| cls == class))
}

/// Get the parent element's node ID.
#[must_use]
pub fn parent_element(html: &Html, node_id: NodeId) -> Option<NodeId> {
    let node_ref = html.tree.get(node_id)?;
    let parent = node_ref.parent()?;
    if matches!(parent.value(), Node::Element(_)) {
        Some(parent.id())
    } else {
        None
    }
}

/// Check if `ancestor_id` is an ancestor of `node_id`.
#[must_use]
pub fn is_ancestor(html: &Html, node_id: NodeId, ancestor_id: NodeId) -> bool {
    let mut current = node_id;
    loop {
        let Some(node_ref) = html.tree.get(current) else {
            return false;
        };
        let Some(parent) = node_ref.parent() else {
            return false;
        };
        if parent.id() == ancestor_id {
            return true;
        }
        current = parent.id();
    }
}

/// Get all child element node IDs.
#[must_use]
pub fn child_elements(html: &Html, node_id: NodeId) -> Vec<NodeId> {
    let Some(node_ref) = html.tree.get(node_id) else {
        return Vec::new();
    };
    node_ref
        .children()
        .filter(|c| matches!(c.value(), Node::Element(_)))
        .map(|c| c.id())
        .collect()
}

/// Compute link density: ratio of text inside `<a>` tags to total text.
#[must_use]
pub fn link_density(html: &Html, node_id: NodeId) -> f64 {
    let total_text = text_content(html, node_id);
    link_density_with_text(html, node_id, &total_text)
}

/// Compute link density using pre-computed total text to avoid
/// redundant tree walks.
#[must_use]
pub fn link_density_with_text(html: &Html, node_id: NodeId, total_text: &str) -> f64 {
    let total_len = total_text.trim().len();
    if total_len == 0 {
        return 0.0;
    }
    let mut link_len = 0usize;
    for a_id in descendant_elements_by_tag(html, node_id, "a") {
        link_len += text_content(html, a_id).trim().len();
    }
    #[expect(clippy::cast_precision_loss)]
    let ratio = link_len as f64 / total_len as f64;
    ratio
}

/// Find all descendant elements with a specific tag.
#[must_use]
pub fn descendant_elements_by_tag(html: &Html, node_id: NodeId, tag: &str) -> Vec<NodeId> {
    let mut result = Vec::new();
    collect_descendants_by_tag(html, node_id, tag, &mut result);
    result
}

fn collect_descendants_by_tag(html: &Html, node_id: NodeId, tag: &str, result: &mut Vec<NodeId>) {
    let Some(node_ref) = html.tree.get(node_id) else {
        return;
    };
    for child in node_ref.children() {
        if let Node::Element(el) = child.value()
            && el.name.local.as_ref() == tag
        {
            result.push(child.id());
        }
        collect_descendants_by_tag(html, child.id(), tag, result);
    }
}

/// Get all descendant element IDs.
#[must_use]
pub fn all_descendant_elements(html: &Html, node_id: NodeId) -> Vec<NodeId> {
    let mut result = Vec::new();
    collect_all_descendants(html, node_id, &mut result);
    result
}

fn collect_all_descendants(html: &Html, node_id: NodeId, result: &mut Vec<NodeId>) {
    let Some(node_ref) = html.tree.get(node_id) else {
        return;
    };
    for child in node_ref.children() {
        if matches!(child.value(), Node::Element(_)) {
            result.push(child.id());
        }
        collect_all_descendants(html, child.id(), result);
    }
}

/// Check if any descendant of `node_id` matches a CSS selector.
#[must_use]
pub fn has_descendant_matching(html: &Html, node_id: NodeId, selector_str: &str) -> bool {
    let Ok(sel) = Selector::parse(selector_str) else {
        return false;
    };
    html.select(&sel)
        .any(|el| is_ancestor(html, el.id(), node_id))
}

/// Check if the element itself matches a CSS selector.
#[must_use]
pub fn element_matches(html: &Html, node_id: NodeId, selector_str: &str) -> bool {
    let Ok(sel) = Selector::parse(selector_str) else {
        return false;
    };
    html.select(&sel).any(|el| el.id() == node_id)
}

/// Check if element or any ancestor matches a CSS selector.
#[must_use]
pub fn self_or_ancestor_matches(html: &Html, node_id: NodeId, selector_str: &str) -> bool {
    let Ok(sel) = Selector::parse(selector_str) else {
        return false;
    };
    let matching_ids: Vec<NodeId> = html.select(&sel).map(|el| el.id()).collect();
    let mut current = Some(node_id);
    while let Some(id) = current {
        if matching_ids.contains(&id) {
            return true;
        }
        current = parent_element(html, id);
    }
    false
}

/// Collect href attribute values from descendant `<a>` elements.
#[must_use]
pub fn collect_link_hrefs(html: &Html, node_id: NodeId) -> Vec<String> {
    let mut hrefs = Vec::new();
    for a_id in descendant_elements_by_tag(html, node_id, "a") {
        if let Some(href) = get_attr(html, a_id, "href") {
            hrefs.push(href);
        }
    }
    hrefs
}

/// Generate a CSS selector path for a node (for debug output).
#[must_use]
pub fn selector_path(html: &Html, node_id: NodeId) -> String {
    let mut parts = Vec::new();
    let mut current = Some(node_id);
    while let Some(id) = current {
        let Some(node_ref) = html.tree.get(id) else {
            break;
        };
        if let Node::Element(el) = node_ref.value() {
            let tag = el.name.local.as_ref();
            let mut part = tag.to_string();
            if let Some(id_attr) = el.attr("id") {
                let _ = write!(part, "#{id_attr}");
            } else if let Some(class_attr) = el.attr("class") {
                let first_class = class_attr.split_whitespace().next();
                if let Some(cls) = first_class {
                    let _ = write!(part, ".{cls}");
                }
            }
            parts.push(part);
        }
        current = node_ref
            .parent()
            .filter(|p| matches!(p.value(), Node::Element(_)))
            .map(|p| p.id());
    }
    parts.reverse();
    parts.join(" > ")
}
