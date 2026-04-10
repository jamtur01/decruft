//! Remove metadata blocks adjacent to h1 elements.
//!
//! Targets short elements (author + date containers) that appear as
//! siblings immediately after the main heading. This complements the
//! inline metadata-div removal in `patterns.rs`.

use ego_tree::NodeId;
use scraper::{Html, Node};

use crate::dom;
use crate::patterns;

/// Remove a date-containing metadata block that is a direct sibling
/// of the first h1 in the content.
///
/// Checks up to 3 siblings after the h1. A match requires a short
/// text element (< 300 chars) containing a recognizable date pattern,
/// either directly or within a child `<p>` or `<time>` element.
pub fn remove_metadata_block(html: &mut Html, main_content: NodeId) {
    let h1_ids = dom::select_within(html, main_content, "h1");
    let Some(&h1_id) = h1_ids.first() else {
        return;
    };

    let target = find_metadata_sibling(html, h1_id);
    if let Some(id) = target {
        dom::remove_node(html, id);
    }
}

/// Walk up to 3 next siblings of the h1, looking for a short
/// date-containing element.
fn find_metadata_sibling(html: &Html, h1_id: NodeId) -> Option<NodeId> {
    let mut sibling_id = next_element_sibling(html, h1_id)?;

    for _ in 0..3 {
        let text = dom::text_content(html, sibling_id);
        let trimmed = text.trim();

        if !trimmed.is_empty() && trimmed.len() < 300 && has_date(html, sibling_id, trimmed) {
            return Some(sibling_id);
        }

        match next_element_sibling(html, sibling_id) {
            Some(next) => sibling_id = next,
            None => return None,
        }
    }

    None
}

/// Check whether the element or its `<p>`/`<time>` children contain a
/// date.
fn has_date(html: &Html, node_id: NodeId, text: &str) -> bool {
    if patterns::is_date_metadata_block(text) {
        return true;
    }
    for sel in &["p", "time"] {
        let child_ids = dom::select_within(html, node_id, sel);
        for &cid in &child_ids {
            let child_text = dom::text_content(html, cid);
            if patterns::is_date_metadata_block(child_text.trim()) {
                return true;
            }
        }
    }
    false
}

/// Find the next element sibling (skipping text nodes).
fn next_element_sibling(html: &Html, node_id: NodeId) -> Option<NodeId> {
    let node_ref = html.tree.get(node_id)?;
    let mut sibling = node_ref.next_sibling();
    while let Some(sib) = sibling {
        if matches!(sib.value(), Node::Element(_)) {
            return Some(sib.id());
        }
        sibling = sib.next_sibling();
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_date_block_after_h1() {
        let doc = r#"<html><body>
            <h1>Article Title</h1>
            <div class="meta">By Author · March 24, 2026</div>
            <p>Article content here</p>
        </body></html>"#;
        let mut html = Html::parse_document(doc);
        let body_ids = dom::select_ids(&html, "body");
        let body = body_ids[0];
        remove_metadata_block(&mut html, body);
        let result = dom::inner_html(&html, body);
        assert!(!result.contains("March 24, 2026"));
        assert!(result.contains("Article content here"));
    }

    #[test]
    fn removes_date_in_child_time_element() {
        let doc = r#"<html><body>
            <h1>Title</h1>
            <div><time datetime="2026-01-15">January 15, 2026</time></div>
            <p>Content</p>
        </body></html>"#;
        let mut html = Html::parse_document(doc);
        let body_ids = dom::select_ids(&html, "body");
        remove_metadata_block(&mut html, body_ids[0]);
        let result = dom::inner_html(&html, body_ids[0]);
        assert!(!result.contains("January 15, 2026"));
        assert!(result.contains("Content"));
    }

    #[test]
    fn preserves_content_after_h1_without_date() {
        let doc = r"<html><body>
            <h1>Title</h1>
            <p>Introduction paragraph</p>
            <p>More content</p>
        </body></html>";
        let mut html = Html::parse_document(doc);
        let body_ids = dom::select_ids(&html, "body");
        remove_metadata_block(&mut html, body_ids[0]);
        let result = dom::inner_html(&html, body_ids[0]);
        assert!(result.contains("Introduction paragraph"));
    }

    #[test]
    fn skips_long_elements() {
        let long_text = "a".repeat(400);
        let doc = format!(
            r"<html><body>
            <h1>Title</h1>
            <div>{long_text} January 1, 2026</div>
            <p>Content</p>
        </body></html>"
        );
        let mut html = Html::parse_document(&doc);
        let body_ids = dom::select_ids(&html, "body");
        remove_metadata_block(&mut html, body_ids[0]);
        let result = dom::inner_html(&html, body_ids[0]);
        assert!(result.contains("January 1, 2026"));
    }

    #[test]
    fn no_h1_is_safe() {
        let doc = "<html><body><p>No heading here</p></body></html>";
        let mut html = Html::parse_document(doc);
        let body_ids = dom::select_ids(&html, "body");
        remove_metadata_block(&mut html, body_ids[0]);
        let result = dom::inner_html(&html, body_ids[0]);
        assert!(result.contains("No heading here"));
    }
}
