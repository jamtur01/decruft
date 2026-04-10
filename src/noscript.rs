use markup5ever::{QualName, ns};
use scraper::{Html, Node};

use crate::dom;

/// Tiny base64 data URIs and empty srcs used as lazy-load placeholders.
const PLACEHOLDER_PREFIXES: &[&str] = &[
    "data:image/gif;base64,R0lGOD",
    "data:image/png;base64,iVBOR",
    "data:image/svg+xml",
    "data:image/webp",
];

/// Resolve noscript images by promoting them over lazy-load
/// placeholders.
///
/// Many pages use a pattern where the real `<img>` is inside a
/// `<noscript>` block and a neighbouring `<img>` has a tiny
/// placeholder `src` plus a `data-src` with the real URL. This
/// function finds those pairs and copies the noscript image's `src`
/// onto the placeholder, then removes the `<noscript>` element.
pub fn resolve_noscript_images(html: &mut Html) {
    let noscript_ids = dom::select_ids(html, "noscript");

    // Collect replacement pairs first to avoid mutating while
    // iterating.
    let replacements: Vec<_> = noscript_ids
        .iter()
        .filter_map(|&noscript_id| build_replacement(html, noscript_id))
        .collect();

    for (placeholder_id, src, noscript_id) in replacements {
        apply_replacement(html, placeholder_id, &src);
        dom::remove_node(html, noscript_id);
    }
}

/// For a single `<noscript>`, find its inner `<img>` src and a
/// neighbouring placeholder `<img>`, returning the triple needed for
/// replacement.
fn build_replacement(
    html: &Html,
    noscript_id: ego_tree::NodeId,
) -> Option<(ego_tree::NodeId, String, ego_tree::NodeId)> {
    let noscript_img_src = find_noscript_img_src(html, noscript_id)?;
    let placeholder = find_placeholder_sibling(html, noscript_id)?;
    Some((placeholder, noscript_img_src, noscript_id))
}

/// Parse the text content of a `<noscript>` element looking for an
/// `<img>` with a `src` attribute.
fn find_noscript_img_src(html: &Html, noscript_id: ego_tree::NodeId) -> Option<String> {
    // scraper parses <noscript> children as text, so we need to
    // re-parse the inner text as HTML to find <img> elements.
    let inner_text = dom::text_content(html, noscript_id);
    if inner_text.trim().is_empty() {
        return None;
    }

    // Also check direct child elements (some parsers keep them).
    for child_id in dom::descendant_elements_by_tag(html, noscript_id, "img") {
        if let Some(src) = dom::get_attr(html, child_id, "src")
            && !src.is_empty()
            && !is_placeholder_src(&src)
        {
            return Some(src);
        }
    }

    // Re-parse the raw text as HTML. html5ever treats noscript
    // contents as raw text (scripting-enabled mode), so inner_html
    // would escape the markup. Use the text content directly.
    let fragment = Html::parse_fragment(&inner_text);
    for img_id in dom::select_ids(&fragment, "img") {
        if let Some(src) = dom::get_attr(&fragment, img_id, "src")
            && !src.is_empty()
            && !is_placeholder_src(&src)
        {
            return Some(src);
        }
    }

    None
}

/// Walk previous and next siblings of a `<noscript>` looking for a
/// placeholder `<img>`.
fn find_placeholder_sibling(
    html: &Html,
    noscript_id: ego_tree::NodeId,
) -> Option<ego_tree::NodeId> {
    let node_ref = html.tree.get(noscript_id)?;

    // Check previous siblings (up to 3 elements away).
    if let Some(found) = find_in_previous_siblings(html, &node_ref) {
        return Some(found);
    }

    // Check next siblings (up to 3 elements away).
    find_in_next_siblings(html, &node_ref)
}

/// Search up to 3 previous siblings for a placeholder image.
fn find_in_previous_siblings(
    html: &Html,
    node_ref: &ego_tree::NodeRef<scraper::Node>,
) -> Option<ego_tree::NodeId> {
    let mut prev = node_ref.prev_sibling();
    for _ in 0..3 {
        let Some(sibling) = prev else { break };
        if let Node::Element(_) = sibling.value()
            && let Some(id) = check_placeholder_img(html, sibling.id())
        {
            return Some(id);
        }
        prev = sibling.prev_sibling();
    }
    None
}

/// Search up to 3 next siblings for a placeholder image.
fn find_in_next_siblings(
    html: &Html,
    node_ref: &ego_tree::NodeRef<scraper::Node>,
) -> Option<ego_tree::NodeId> {
    let mut next = node_ref.next_sibling();
    for _ in 0..3 {
        let Some(sibling) = next else { break };
        if let Node::Element(_) = sibling.value()
            && let Some(id) = check_placeholder_img(html, sibling.id())
        {
            return Some(id);
        }
        next = sibling.next_sibling();
    }
    None
}

/// If the node is a placeholder `<img>` (or contains one), return its
/// `NodeId`.
fn check_placeholder_img(html: &Html, node_id: ego_tree::NodeId) -> Option<ego_tree::NodeId> {
    if dom::is_tag(html, node_id, "img") && is_placeholder(html, node_id) {
        return Some(node_id);
    }

    // Check if a child <img> is a placeholder (e.g. wrapped in a
    // <span> or <div>).
    dom::descendant_elements_by_tag(html, node_id, "img")
        .into_iter()
        .find(|&child_id| is_placeholder(html, child_id))
}

/// Determine whether an `<img>` element is a lazy-load placeholder.
fn is_placeholder(html: &Html, img_id: ego_tree::NodeId) -> bool {
    // Has data-src => definitely a lazy-load placeholder.
    if dom::get_attr(html, img_id, "data-src").is_some() {
        return true;
    }

    let src = dom::get_attr(html, img_id, "src").unwrap_or_default();
    is_placeholder_src(&src)
}

/// Check if a `src` value looks like a placeholder (empty, tiny
/// base64, or blank pixel).
fn is_placeholder_src(src: &str) -> bool {
    let trimmed = src.trim();
    if trimmed.is_empty() || trimmed == "#" || trimmed == "about:blank" {
        return true;
    }
    for prefix in PLACEHOLDER_PREFIXES {
        if trimmed.starts_with(prefix) {
            return true;
        }
    }
    false
}

/// Set the `src` attribute on a placeholder `<img>` element.
fn apply_replacement(html: &mut Html, img_id: ego_tree::NodeId, src: &str) {
    let Some(mut node) = html.tree.get_mut(img_id) else {
        return;
    };
    let Node::Element(el) = node.value() else {
        return;
    };
    let qn = QualName::new(None, ns!(), markup5ever::LocalName::from("src"));
    el.attrs.retain(|(n, _)| n != &qn);
    el.attrs
        .push((qn, markup5ever::tendril::StrTendril::from(src)));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn promotes_noscript_img_over_placeholder() {
        let html_str = r#"<html><body>
            <img src="data:image/gif;base64,R0lGODlhAQABAAAA" data-src="lazy.jpg">
            <noscript><img src="real.jpg"></noscript>
        </body></html>"#;
        let mut doc = Html::parse_document(html_str);
        resolve_noscript_images(&mut doc);
        let output = dom::outer_html(&doc, doc.tree.root().id());
        assert!(output.contains(r#"src="real.jpg""#));
        assert!(!output.contains("noscript"));
    }

    #[test]
    fn ignores_noscript_without_img() {
        let html_str = r"<html><body>
            <noscript><p>Enable JavaScript</p></noscript>
        </body></html>";
        let mut doc = Html::parse_document(html_str);
        resolve_noscript_images(&mut doc);
        let output = dom::outer_html(&doc, doc.tree.root().id());
        assert!(output.contains("noscript"));
    }

    #[test]
    fn ignores_noscript_without_nearby_placeholder() {
        let html_str = r#"<html><body>
            <div><img src="normal.jpg"></div>
            <p>some text</p>
            <p>more text</p>
            <p>even more text</p>
            <p>far away</p>
            <noscript><img src="real.jpg"></noscript>
        </body></html>"#;
        let mut doc = Html::parse_document(html_str);
        resolve_noscript_images(&mut doc);
        let output = dom::outer_html(&doc, doc.tree.root().id());
        // noscript should remain since normal.jpg is not a placeholder
        assert!(output.contains("noscript"));
    }

    #[test]
    fn handles_empty_src_placeholder() {
        let html_str = r#"<html><body>
            <img src="">
            <noscript><img src="real.jpg"></noscript>
        </body></html>"#;
        let mut doc = Html::parse_document(html_str);
        resolve_noscript_images(&mut doc);
        let output = dom::outer_html(&doc, doc.tree.root().id());
        assert!(output.contains(r#"src="real.jpg""#));
        assert!(!output.contains("noscript"));
    }

    #[test]
    fn promotes_noscript_img_to_next_sibling_placeholder() {
        let html_str = r#"<html><body>
            <noscript><img src="real.jpg"></noscript>
            <img src="" data-src="lazy.jpg">
        </body></html>"#;
        let mut doc = Html::parse_document(html_str);
        resolve_noscript_images(&mut doc);
        let output = dom::outer_html(&doc, doc.tree.root().id());
        assert!(output.contains(r#"src="real.jpg""#));
        assert!(!output.contains("noscript"));
    }

    #[test]
    fn placeholder_src_detection() {
        assert!(is_placeholder_src(""));
        assert!(is_placeholder_src("#"));
        assert!(is_placeholder_src("about:blank"));
        assert!(is_placeholder_src(
            "data:image/gif;base64,R0lGODlhAQABAAAAACH5BAEKAAEALAAAAAABAAEAAAICTAEAOw=="
        ));
        assert!(is_placeholder_src("data:image/svg+xml;base64,abc"));
        assert!(!is_placeholder_src("https://example.com/image.jpg"));
        assert!(!is_placeholder_src("/images/photo.png"));
    }
}
